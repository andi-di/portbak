const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

function parseVersion(value) {
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(value)) {
    throw new Error(`Invalid release version: ${value}`);
  }
  const parts = value.split('.').map(Number);
  if (parts.some(part => part > 65535)) {
    throw new Error('Release version exceeds Windows version metadata limits');
  }
  return parts;
}

function compareVersions(left, right) {
  const a = parseVersion(left);
  const b = parseVersion(right);
  for (let index = 0; index < 3; index++) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return 0;
}

function validateVersion(version, tags, mergeSha) {
  parseVersion(version);
  const tag = `v${version}`;
  const existing = tags.find(candidate => candidate.name === tag);
  if (existing) {
    if (mergeSha && existing.commit.sha === mergeSha) return tag;
    throw new Error(`Version ${version} already has a release tag; bump Cargo.toml and Cargo.lock`);
  }
  const versions = tags.filter(candidate => /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(candidate.name));
  if (versions.some(candidate => compareVersions(version, candidate.name.slice(1)) <= 0)) {
    throw new Error('Cargo.toml version must be newer than every existing release tag');
  }
  return tag;
}

async function validateReleaseVersion(github, repo, version, mergeSha) {
  const tags = await github.paginate(github.rest.repos.listTags, { ...repo, per_page: 100 });
  return validateVersion(version, tags, mergeSha);
}

function cargoVersion(manifest) {
  const section = manifest.split(/(?=^\[)/m).find(section => section.startsWith('[package]'));
  const version = section?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  parseVersion(version);
  return version;
}

function mergedPullRequest(context) {
  const pr = context.payload.pull_request;
  if (context.payload.action !== 'closed' || !pr?.merged || pr.base.ref !== 'main') {
    throw new Error('Releases require a PR merged into main');
  }
  if (!/^[a-f0-9]{40}$/.test(pr.merge_commit_sha)) {
    throw new Error('Missing merge commit');
  }
  return pr;
}

async function releaseContext(github, context) {
  if (context.eventName !== 'push') return context;
  if (context.ref !== 'refs/heads/main' || context.payload.deleted || !/^[a-f0-9]{40}$/.test(context.sha)) {
    throw new Error('Releases require a live push to main');
  }
  const pulls = await github.paginate(github.rest.repos.listPullRequestsAssociatedWithCommit, {
    ...context.repo, commit_sha: context.sha, per_page: 100,
  });
  const matches = pulls.filter(pr => pr.merged_at && pr.base.ref === 'main'
    && pr.merge_commit_sha === context.sha);
  if (matches.length > 1) throw new Error('Multiple merged PRs match the pushed commit');
  if (!matches.length) return null;
  return { ...context, repo: context.repo, payload: {
    action: 'closed', pull_request: { ...matches[0], merged: true },
  } };
}

async function releaseByTag(github, repo, tag) {
  try {
    return (await github.rest.repos.getReleaseByTag({ ...repo, tag })).data;
  } catch (error) {
    if (error.status !== 404) throw error;
    // The tag endpoint returns published releases; authenticated listings also include drafts.
    const releases = await github.paginate(github.rest.repos.listReleases, { ...repo, per_page: 100 });
    const matches = releases.filter(release => release.tag_name === tag);
    if (matches.length > 1) throw new Error(`Multiple releases reserve ${tag}`);
    return matches[0] ?? null;
  }
}

async function prepareRelease({ github, context, core }, version) {
  context = await releaseContext(github, context);
  if (!context) {
    core.setOutput('published', 'true');
    core.info?.('No merged PR introduced this commit; skipping release.');
    return { published: 'true' };
  }
  const pr = mergedPullRequest(context);
  const repo = context.repo;
  const tags = await github.paginate(github.rest.repos.listTags, { ...repo, per_page: 100 });
  const tag = validateVersion(version, tags, pr.merge_commit_sha);
  if (!tags.some(candidate => candidate.name === tag)) {
    await github.rest.git.createRef({ ...repo, ref: `refs/tags/${tag}`, sha: pr.merge_commit_sha });
  }
  let release = await releaseByTag(github, repo, tag);
  if (!release) {
    release = (await github.rest.repos.createRelease({
      ...repo, tag_name: tag, target_commitish: pr.merge_commit_sha,
      name: `portbak ${tag}`, draft: true, prerelease: false,
      generate_release_notes: true,
      body: `Release for merged PR #${pr.number}: ${pr.html_url}`,
    })).data;
  }
  const outputs = {
    version, tag, sha: pr.merge_commit_sha, release_id: String(release.id),
    published: String(!release.draft),
  };
  for (const [name, value] of Object.entries(outputs)) core.setOutput(name, value);
  return outputs;
}

function releaseFiles(directory, version) {
  parseVersion(version);
  const names = [
    `portbak-v${version}-linux-x86_64.tar.gz`,
    `portbak-v${version}-windows-x86_64.zip`,
  ];
  const files = names.map(name => ({ name, data: fs.readFileSync(path.join(directory, name)) }));
  const checksums = files.map(file =>
    `${crypto.createHash('sha256').update(file.data).digest('hex')}  ${file.name}\n`).join('');
  files.push({ name: 'SHA256SUMS', data: Buffer.from(checksums) });
  return files;
}

async function publishRelease({ github, context }, { version, directory }) {
  context = await releaseContext(github, context);
  if (!context) throw new Error('No merged PR introduced this commit');
  const pr = mergedPullRequest(context);
  const tag = `v${version}`;
  parseVersion(version);
  const repo = context.repo;
  const ref = (await github.rest.git.getRef({ ...repo, ref: `tags/${tag}` })).data;
  if (ref.object.type !== 'commit' || ref.object.sha !== pr.merge_commit_sha) {
    throw new Error('Release tag does not point to the expected merge commit');
  }
  const release = await releaseByTag(github, repo, tag);
  if (!release) throw new Error('Release reservation is missing');
  if (!release.draft) return;
  // Read both archives before uploading anything; a missing platform must block publication.
  const files = releaseFiles(directory, version);
  const assets = await github.paginate(github.rest.repos.listReleaseAssets, {
    ...repo, release_id: release.id, per_page: 100,
  });
  if (assets.some(asset => !files.some(file => file.name === asset.name))) {
    throw new Error('Unexpected draft assets; refusing to publish');
  }
  for (const file of files) {
    const previous = assets.find(asset => asset.name === file.name);
    // Only unpublished draft assets may be replaced after an interrupted upload.
    if (previous) await github.rest.repos.deleteReleaseAsset({ ...repo, asset_id: previous.id });
    await github.rest.repos.uploadReleaseAsset({
      ...repo, release_id: release.id, name: file.name, data: file.data,
      headers: { 'content-type': 'application/octet-stream' },
    });
  }
  await github.rest.repos.updateRelease({
    ...repo, release_id: release.id, draft: false, prerelease: false,
    // Retrying an older failed release must not replace the newest release's download link.
    make_latest: 'legacy',
  });
}

module.exports = { parseVersion, validateVersion, validateReleaseVersion, cargoVersion, mergedPullRequest, prepareRelease,
  releaseFiles, publishRelease };

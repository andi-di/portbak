const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const crypto = require('node:crypto');
const { bumpVersion, cargoVersion, parseVersion, prepareRelease, releaseFiles,
  publishRelease } = require('./release.cjs');

const SHA = 'a'.repeat(40);
function context(sha = SHA, labels = []) {
  return { repo: { owner: 'owner', repo: 'portbak' }, payload: {
    action: 'closed', pull_request: { merged: true, base: { ref: 'main' },
      merge_commit_sha: sha, labels: labels.map(name => ({ name })),
      number: 42, html_url: 'https://github.com/owner/portbak/pull/42' },
  } };
}

class FakeGitHub {
  tags = [];
  releases = [];
  assets = [];
  writes = [];
  failUpload = '';
  rest = {
    repos: {
      listTags() {},
      listReleaseAssets() {},
      getReleaseByTag: async ({ tag }) => {
        const data = this.releases.find(release => release.tag_name === tag);
        if (!data) throw Object.assign(new Error('Missing'), { status: 404 });
        return { data };
      },
      createRelease: async (input) => {
        const data = { ...input, id: this.releases.length + 1 };
        this.releases.push(data);
        this.writes.push(['createRelease', input]);
        return { data };
      },
      deleteReleaseAsset: async ({ asset_id }) => {
        this.writes.push(['deleteReleaseAsset', asset_id]);
        this.assets = this.assets.filter(asset => asset.id !== asset_id);
      },
      uploadReleaseAsset: async (input) => {
        if (input.name === this.failUpload) throw new Error('Upload failed');
        this.assets.push({ ...input, id: this.assets.length + 1 });
        this.writes.push(['uploadReleaseAsset', input]);
        return { data: input };
      },
      updateRelease: async (input) => {
        Object.assign(this.releases.find(release => release.id === input.release_id), input);
        this.writes.push(['updateRelease', input]);
      },
    },
    git: {
      createRef: async ({ ref, sha }) => {
        assert(!this.tags.some(tag => `refs/tags/${tag.name}` === ref));
        this.tags.push({ name: ref.slice('refs/tags/'.length), commit: { sha } });
        this.writes.push(['createRef', ref]);
      },
      getRef: async ({ ref }) => ({ data: {
        object: { type: 'commit', sha: this.tags.find(tag => `tags/${tag.name}` === ref)?.commit.sha },
      } }),
    },
  };
  paginate = async (method) => {
    if (method === this.rest.repos.listTags) return this.tags;
    if (method === this.rest.repos.listReleaseAssets) return this.assets;
    throw new Error('Unexpected pagination');
  };
}

const core = { setOutput() {} };
async function prepared() {
  const github = new FakeGitHub();
  await prepareRelease({ github, context: context(), core }, '0.1.0');
  return github;
}

function archives(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'portbak-release-test-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(path.join(directory, 'portbak-v0.2.0-linux-x86_64.tar.gz'), 'linux archive');
  fs.writeFileSync(path.join(directory, 'portbak-v0.2.0-windows-x86_64.zip'), 'windows archive');
  return directory;
}

test('minor is the default; explicit patch and major labels reset the right components', () => {
  assert.equal(bumpVersion('0.1.0', []), '0.2.0');
  assert.equal(bumpVersion('1.9.4', ['bug', 'release:minor']), '1.10.0');
  assert.equal(bumpVersion('1.9.4', ['release:patch']), '1.9.5');
  assert.equal(bumpVersion('1.9.4', ['release:major']), '2.0.0');
  assert.throws(() => bumpVersion('1.9.4', ['release:major', 'release:patch']), /at most one/);
  assert.throws(() => bumpVersion('1.9.4', ['release:rc']), /at most one/);
});

test('only stable versions representable in Windows metadata are accepted', () => {
  for (const value of ['v1.0.0', '1.0', '1.0.0-rc.1', '01.0.0', '65536.0.0', '1.2.3\n']) {
    assert.throws(() => parseVersion(value), undefined, value);
  }
  assert.throws(() => bumpVersion('0.65535.0', []), /Windows/);
  assert.equal(cargoVersion('[package]\nname = "portbak"\nversion = "0.1.0"\n[dependencies]\nx = "2"\n'), '0.1.0');
});

test('nonmerged PRs and other branches cannot reserve releases', async () => {
  for (const mutate of [c => c.payload.pull_request.merged = false,
    c => c.payload.pull_request.base.ref = 'feature', c => c.payload.action = 'opened']) {
    const github = new FakeGitHub();
    const input = context();
    mutate(input);
    await assert.rejects(prepareRelease({ github, context: input, core }, '0.1.0'), /merged/);
    assert.equal(github.writes.length, 0);
  }
});

test('conflicting labels fail before any repository mutation', async () => {
  const github = new FakeGitHub();
  await assert.rejects(prepareRelease({ github,
    context: context(SHA, ['release:major', 'release:patch']), core }, '0.1.0'), /at most one/);
  assert.deepEqual(github.writes, []);
});

test('API errors are not treated as missing releases', async () => {
  const github = new FakeGitHub();
  github.tags.push({ name: 'v0.2.0', commit: { sha: SHA } });
  github.rest.repos.getReleaseByTag = async () => {
    throw Object.assign(new Error('Forbidden'), { status: 403 });
  };
  await assert.rejects(prepareRelease({ github, context: context(), core }, '0.1.0'), /Forbidden/);
  assert.deepEqual(github.writes, []);
});

test('reservation uses the highest numeric tag and binds a draft to the exact merge', async () => {
  const github = new FakeGitHub();
  github.tags = ['v0.9.0', 'v0.10.0', 'v2.0.0-rc.1', 'unrelated'].map(name =>
    ({ name, commit: { sha: 'b'.repeat(40) } }));
  const result = await prepareRelease({ github, context: context(), core }, '0.1.0');
  assert.equal(result.version, '0.11.0');
  assert.equal(result.sha, SHA);
  assert.equal(result.published, 'false');
  assert.equal(github.releases[0].target_commitish, SHA);
  assert.equal(github.releases[0].draft, true);
  assert.equal(github.releases[0].prerelease, false);
  assert.equal(github.releases[0].generate_release_notes, true);
});

test('failed builds keep their version while later PRs receive distinct versions', async () => {
  const github = await prepared();
  await prepareRelease({ github, context: context('b'.repeat(40)), core }, '0.1.0');
  const result = await prepareRelease({ github, context: context(), core }, '0.1.0');
  assert.equal(result.version, '0.2.0');
  assert.deepEqual(github.tags.map(tag => tag.name), ['v0.2.0', 'v0.3.0']);
  assert.equal(github.releases.length, 2);
});

test('a tag left by an interrupted reservation is reused', async () => {
  const github = new FakeGitHub();
  github.tags.push({ name: 'v0.2.0', commit: { sha: SHA } });
  await prepareRelease({ github, context: context(), core }, '0.1.0');
  assert.equal(github.tags.length, 1);
  assert.equal(github.releases.length, 1);
});

test('published releases are skipped without touching their assets', async () => {
  const github = await prepared();
  github.releases[0].draft = false;
  github.writes = [];
  const result = await prepareRelease({ github, context: context(), core }, '0.1.0');
  assert.equal(result.published, 'true');
  await publishRelease({ github, context: context() }, { version: '0.2.0', directory: 'missing' });
  assert.deepEqual(github.writes, []);
});

test('a missing platform archive blocks uploads and publication', async (t) => {
  const directory = archives(t);
  fs.unlinkSync(path.join(directory, 'portbak-v0.2.0-windows-x86_64.zip'));
  const github = await prepared();
  github.writes = [];
  await assert.rejects(publishRelease({ github, context: context() }, { version: '0.2.0', directory }));
  assert.deepEqual(github.writes, []);
  assert.equal(github.releases[0].draft, true);
});

test('interrupted uploads remain drafts and retry safely before publishing', async (t) => {
  const directory = archives(t);
  const github = await prepared();
  github.failUpload = 'portbak-v0.2.0-windows-x86_64.zip';
  await assert.rejects(publishRelease({ github, context: context() }, { version: '0.2.0', directory }), /Upload failed/);
  assert.equal(github.releases[0].draft, true);
  assert.equal(github.assets.length, 1);
  github.failUpload = '';
  await publishRelease({ github, context: context() }, { version: '0.2.0', directory });
  assert.equal(github.releases[0].draft, false);
  assert.equal(github.releases[0].prerelease, false);
  assert.equal(github.releases[0].make_latest, 'legacy');
  assert.equal(github.assets.length, 3);
  const manifest = github.assets.find(asset => asset.name === 'SHA256SUMS').data.toString();
  for (const file of releaseFiles(directory, '0.2.0').slice(0, 2)) {
    assert(manifest.includes(`${crypto.createHash('sha256').update(file.data).digest('hex')}  ${file.name}\n`));
  }
  assert.equal(github.writes.at(-1)[0], 'updateRelease');
});

test('a moved tag blocks publication', async (t) => {
  const github = await prepared();
  github.tags[0].commit.sha = 'b'.repeat(40);
  github.writes = [];
  await assert.rejects(publishRelease({ github, context: context() },
    { version: '0.2.0', directory: archives(t) }), /expected merge/);
  assert.deepEqual(github.writes, []);
});

test('unexpected draft assets block publication without deleting anything', async (t) => {
  const github = await prepared();
  github.assets.push({ name: 'portbak.json', id: 1 });
  github.writes = [];
  await assert.rejects(publishRelease({ github, context: context() },
    { version: '0.2.0', directory: archives(t) }), /Unexpected draft assets/);
  assert.deepEqual(github.writes, []);
  assert.equal(github.releases[0].draft, true);
});

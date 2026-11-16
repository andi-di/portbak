# Releases

Every PR merged into `main` queues an automatic release. Open PRs and closed,
unmerged PRs never publish. Release notes live in GitHub Releases only.

## Release version

Each PR sets the upcoming release version in `Cargo.toml` and updates the
`portbak` entry in `Cargo.lock` to match. Use a stable `major.minor.patch` version
with each component at most 65535, as required by Windows executable metadata.
Choose the bump to match the change: patch for fixes, minor for features, and
major for breaking changes.

The required Validate release version check rejects versions that already have
a tag or are not newer than every existing stable release tag. If another PR
releases your chosen version first, update your PR to a newer version before
merging. Release labels no longer select or increment versions.

Local builds, CI builds, and published executables all use the version from
`Cargo.toml`. Merging publishes that exact version; it does not increment the
manifest or create an automated version-bump commit.

Application versions do not change the settings or backup metadata format versions.

## Publication and retries

The workflow uses pushes to `main` and verifies the associated merged PR;
direct pushes without a matching merged PR do not release. It does not use
`pull_request_target` or require an exception to its execution policy.

The release workflow reserves a version tag on the exact merged commit and an
unpublished draft, then validates and builds both platforms. Only after both
succeed does it attach the Linux archive, Windows archive, and `SHA256SUMS` and
publish the normal release. Build jobs have read-only repository access; version
reservation and publication jobs require `contents: write`.

The Build and test workflow runs for PRs and manual requests. After a merge,
the release workflow performs the checks and builds both platforms, avoiding
a duplicate Build and test run on `main`.

If a build or upload fails, rerun the failed workflow jobs in GitHub Actions.
A version tag can be reused only for the same merged commit. The reserved
version stays associated with its merge commit, even if newer PRs
release first. Incomplete draft uploads can be replaced on retry. Published
assets and tags are never overwritten. Do not delete reserved tags to reuse
their numbers; a failed release can leave a gap until it is retried.

Release archives contain only the executable, app license, embedded font
licenses, and a short usage note. Settings and backups are excluded. Linux
builds use Ubuntu 22.04; this does not establish a glibc 2.28 compatibility claim.

Local checks for the release helpers:

```sh
node --test .github/scripts/release.test.cjs
python -B -m unittest discover -s .github/scripts -p 'test_*.py'
```

The queue uses GitHub Actions' `concurrency.queue: max` so merges wait instead
of replacing a single pending run. Older workflow validators may not recognize
this setting; see [GitHub's concurrency documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency).

# Releases

Every PR merged into `main` queues an automatic release. Open PRs and closed,
unmerged PRs never publish. Release notes live in GitHub Releases only.

The default bump is **minor** (`0.1.0` → `0.2.0`). Set one PR label to override it:

| Label | Effect |
| --- | --- |
| `release:patch` | Increment patch only |
| `release:minor` | Increment minor and reset patch; same as the default |
| `release:major` | Increment major and reset minor and patch |

Conflicting or unknown `release:` labels fail validation. Labels are read from
the merged PR when its main-branch push is processed. `Cargo.toml` supplies the baseline before the first tag and the
version for local development builds. Release builds receive their allocated
version through `PORTBAK_RELEASE_VERSION`; no bot commits are made to `main`.
Application versions do not change the settings or backup metadata format versions.

The workflow uses pushes to `main` and verifies the associated merged PR;
direct pushes without a matching merged PR do not release. It does not use
`pull_request_target` or require an exception to its execution policy.

The release workflow reserves a version tag on the exact merged commit and an
unpublished draft, then validates and builds both platforms. Only after both
succeed does it attach the Linux archive, Windows archive, and `SHA256SUMS` and
publish the normal release. Build jobs have read-only repository access; version
reservation and publication jobs require `contents: write`.

If a build or upload fails, rerun the failed workflow jobs in GitHub Actions.
The reserved version stays associated with its merge commit, even if newer PRs
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

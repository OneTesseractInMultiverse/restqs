# Release Checklist

Use this checklist before publishing a new crate version. See [Publishing](publishing.md) for setup and CLI commands.

## Prepare the Version

- Choose an unpublished version after checking the crates.io version history and applying the
  [compatibility policy](compatibility.md#release-lines), including its `0.x` rules.
- Update `version` in `Cargo.toml` and record user-facing changes in `CHANGELOG.md`.
- Record the latest published comparison baseline (tag and full commit) in the release-preparation PR. For 0.2.0 use
  [the recorded 0.1.1 baseline](compatibility.md#public-api-comparison-baseline).
- Run the pinned API comparisons for the core and all-feature surfaces. Compatible patches must pass. For a new
  incompatible line, review every diagnostic and document each accepted break; distinguish tool/build errors from
  compatibility findings. Do not hide failures by selecting a larger version number or ignoring the exit status.
- Review public struct construction, exhaustive enum matches, trait implementations, plan shape, query acceptance,
  default limits, validation precedence, and stable error-code meanings beyond what the API tool checks.
- Keep the declared library MSRV within the release line; announce an intentional increase in a new incompatible line.
- Add user-facing migration notes with before/after requests or APIs, resulting errors, and client changes for every
  incompatibility or stricter validation change. Explain any narrow security patch exception and disclosure plan.
- Update README, rustdoc, internal documentation, and `/docs` for changed behavior. Run `make doc-internal` and
  review executable guide examples. Remove obsolete design material and duplicate snippets; retain migration/history.
- Confirm package metadata points to the correct public repository and documentation.
- Merge the reviewed changes into `main` before tagging.

## Local Gates

```sh
python3 -m unittest discover -s .github/scripts -p 'test_*.py'
make verify
make coverage
make package-list
make package
cargo publish --dry-run --locked --registry crates-io
```

Use Python 3.11 or newer. `make verify` checks both the default parser and the `sqlx` feature configuration.
`make coverage` requires 100% source line coverage. `make package` compiles the packaged copy.

The package must contain source, tests, examples, README, documentation, policy files, support docs, and the MIT license.
It must exclude build output, editor metadata, credentials, local coverage reports, and release automation.

## Check Publication Settings

- Confirm private vulnerability reporting is enabled and maintainer security notifications are configured using
  [the security policy](../SECURITY.md#maintainer-verification).

- The `crates-io` environment allows only branch `main` and tags `v*`.
- `OneTesseractInMultiverse` is the required reviewer; self-review is allowed for the solo maintainer.
- Administrator bypass of environment protection is disabled.
- The saved crates.io Trusted Publisher matches `OneTesseractInMultiverse/restqs`, `publish.yml`, and `crates-io`.

## Validate and Publish

- Create an actual `vMAJOR.MINOR.PATCH` tag whose version matches the manifest. Prerelease suffixes are also supported.
- Dispatch Publish from `main` with the tag and `publish=false` for a validation-only run.
- Check the resolved SHA in the run summary.
- Confirm stable Rust, Rust 1.85.0, both feature configurations, RustSec, assertion policy, coverage, and package dry-run gates pass.
- Prepare a draft GitHub release with reviewed notes. Publish the draft when ready, then approve its `crates-io`
  deployment after the checks succeed. A draft is release preparation and does not upload the package.
- Confirm the intended version appears on crates.io and docs.rs.
- Record the published tag and full commit as the next API baseline, and update the supported-version policy when
  advancing the maintained minor line. Preserve old tags and migration notes.

All release jobs check out the resolved SHA, including after environment approval. Never move a published release tag.
Do not reuse a published crate version; prepare a new version for follow-up changes. For a failed upload, check crates.io
before using the manual `publish=true` retry flow.

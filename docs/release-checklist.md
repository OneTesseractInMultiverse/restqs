# Release Checklist

Use this checklist before publishing a new crate version. See [Publishing](publishing.md) for setup and CLI commands.

## Prepare the Version

- Choose an unpublished version; `0.1.0` is already on crates.io.
- Update `version` in `Cargo.toml` and record user-facing changes in `CHANGELOG.md`.
- Review public API, plan shape, and `RqsError::error_code()` compatibility.
- Update README and `/docs` for changed behavior.
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

- The `crates-io` environment allows only branch `main` and tags `v*`.
- `OneTesseractInMultiverse` is the required reviewer; self-review is allowed for the solo maintainer.
- Administrator bypass of environment protection is disabled.
- The saved crates.io Trusted Publisher matches `OneTesseractInMultiverse/restqs`, `publish.yml`, and `crates-io`.

## Validate and Publish

- Create an actual `vMAJOR.MINOR.PATCH` tag whose version matches the manifest. Prerelease suffixes are also supported.
- Dispatch Publish from `main` with the tag and `publish=false` for a validation-only run.
- Check the resolved SHA in the run summary.
- Confirm stable Rust, Rust 1.85.0, both feature configurations, RustSec, coverage, and package dry-run gates pass.
- Publish the GitHub release for that tag, then approve its `crates-io` deployment after the checks succeed.
- Confirm the intended version appears on crates.io and docs.rs.

All release jobs check out the resolved SHA, including after environment approval. Never move a published release tag.
Do not reuse a published crate version; prepare a new version for follow-up changes. For a failed upload, check crates.io
before using the manual `publish=true` retry flow.

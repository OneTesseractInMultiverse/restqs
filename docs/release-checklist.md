# Release Checklist

Use this checklist before publishing a new crate version.

Update `version` in `Cargo.toml`. Update `CHANGELOG.md` with user-facing changes. Confirm that repository and
documentation metadata point to the public project locations. Confirm that public API changes appear in `README.md` and
`/docs`.

## Local Gates

Run the local checks:

```sh
make verify
make coverage
make package-list
make package
make publish-dry-run
```

`make verify` checks formatting, type checking, Clippy, tests, doc tests, and docs.rs-style docs. `make coverage` fails
on uncovered source lines.
`make package-list` shows the files shipped to crates.io. `make package`
verifies the packaged copy.

## Package Contents

The package must include:

- Source files.
- Tests.
- Examples.
- README.
- Documentation under `/docs`.
- Policy files.
- Support docs.
- MIT license text.

The package must exclude build output, editor metadata, secrets, local coverage reports, and machine-specific files.

## CI And Release

Confirm CI passes on stable Rust and the declared minimum Rust version. Confirm the RustSec advisory audit passes.
Confirm that the package name is `restqs`.

Use semantic versioning. Patch releases fix behavior without public API changes. Minor releases add compatible APIs.
Major releases can change public API, plan shape, or error contracts.

`RqsError::error_code()` values are compatibility-sensitive. Treat changes to existing codes as breaking changes.

## Tagging

Release tags use `vMAJOR.MINOR.PATCH`, such as `v0.1.0`. The tag version must match `Cargo.toml`.

After the GitHub release is published, the publish workflow can publish the crate through the protected `crates-io`
environment.

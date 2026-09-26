# Contributing

Contributions are welcome. The project favors focused changes with clear tests, clear documentation, and narrow public
contracts.

RestQS has a strict boundary model. The parser produces a typed plan. Adapters translate that plan. Application code
decides authorization and database execution. Contributions need to preserve that split.

## Development Setup

Install a stable Rust toolchain that supports the crate's `rust-version`. Then install local tools:

```sh
make setup
```

Run the local quality gate:

```sh
make verify
```

Run coverage for source-line checks:

```sh
make coverage
```

When changing SQLite repository examples, also run `make verify-sqlite`. This separate, locked fixture uses SQLx and
an in-memory database without external services. See [the testing guide](docs/testing-guide.md#sqlite-execution-checks).

For PostgreSQL/MySQL integration changes, provision disposable services and run `make verify-services` as described in
[the testing guide](docs/testing-guide.md#postgresql-and-mysql-execution-checks). Service tests are intentionally outside
`make verify`; stable CI runs them explicitly, and `make audit` checks every fixture lockfile.

For parser or adapter changes, run `make verify-properties`. Use `make fuzz-setup` and `make fuzz-smoke` for bounded
instrumented checks. Preserve minimized failures as described in [the testing guide](docs/testing-guide.md#generated-properties-and-fuzzing).

## Contribution Rules

Keep changes focused and small. Preserve the coordinator-versus-computation split described in `docs/architecture.md`.
Add or update tests for every behavior change. Update docs for public API, examples, error codes, adapter behavior, or
security behavior changes.

Unit tests stay free of external configuration. They do not require files, network access, local services, databases,
credentials, or process-specific environment variables.

Each test function uses one assertion. Helper functions return data or errors and contain no assertions. The final
assertion checks one fact. `make test-policy` checks Rust/Python syntax, including feature-gated and ignored tests;
`make verify` includes the checker and its regression tests. Generated test macros need explicit checker support.
See [the precise policy and scope](docs/testing-guide.md#one-assertion-rule).

Explain each new dependency. Include its purpose, maintenance cost, license, release activity, RustSec status, and role
in parser or adapter behavior.

## Pull Request Checklist

Open an issue before a large API change. Use the issue templates for bugs and feature requests. Use GitHub Discussions
for usage questions or broad design discussion.

Run these commands before opening a pull request:

```sh
make fmt
make lint
make test
make doc
make verify-test-policy
make coverage
make package
```

The code must compile without unsafe code and without Clippy warnings.

## Review Focus

Reviewers check behavior, safety boundaries, tests, and documentation. A change that turns raw field names into SQL
identifiers needs revision. A change that mixes parsing with SQL execution needs revision. A change that adds an adapter
without dialect-specific tests needs revision.

Public error codes are compatibility-sensitive. Treat changes to existing codes as public API changes.

# Parser and adapter properties

This unpublished crate shares pure contract predicates with the fuzz targets. It keeps proptest out of the library and
the ordinary unit suite. Run from the repository root:

```sh
make verify-properties
make CARGO='cargo +1.85.0' verify-properties
PROPTEST_CASES=10000 PROPTEST_RNG_SEED=2026 make test-properties
```

The default run uses 256 cases per property, seed 5394771, and at most 4096 shrinking iterations. Explicit proptest
environment settings can increase cases or change the seed for longer local runs. Generated strings contain up to 255
arbitrary Unicode characters; structured queries force successful parsing alongside malformed arbitrary text. Numeric,
list, syntax-path, and limit generators remain bounded. Both Rust CI jobs run the reproducible defaults.

Separate properties check panic freedom, percent decoding, catalog identity/metadata, operator/value compatibility,
raw query bytes, nonempty parameter count, decoded value bytes across scalar/wrapper/regex/list/control paths, list item
bounds, complete bind contents, placeholder correspondence, and SQL structure remaining identical when values change.
Each test contains one assertion. The framework captures panics and shrinks failing cases; helper predicates contain no
assertions and do not execute databases.

Proptest saves failing seeds to `regressions/seeds.txt`. Preserve them when fixing a failure and add the minimized raw
query plus a focused expected-result test to `corpus/` and `tests/regressions.rs`. The initial corpus replays previously
fixed operator-in-value, malformed encoding, ordered-list, non-finite float, regex-flag, duplicate-control, and control
size-limit cases. Compile-time inclusion makes those regressions independent of runtime filesystem setup.

Proptest 1.11.0 (MIT OR Apache-2.0) is a maintained testing library providing bounded generation, shrinking, deterministic
seeds, and failure persistence. It is a development-only dependency of this isolated fixture. Maintenance consists of
lockfile/update review, compatibility checks, and promoting failures into focused regressions. CI audits its committed
lockfile and Dependabot checks it weekly. The shared harness itself depends only on RestQS.

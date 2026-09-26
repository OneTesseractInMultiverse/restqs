# Bounded fuzzing

Three targets exercise percent decoding, parsing under varied limits, and successful SQL adapter translation. The
adapter target also checks that arbitrary value text cannot change SQL structure. They share predicates with the
property fixture and accept at most 4096 input bytes, converting arbitrary bytes to valid UTF-8 with replacement. This
still permits malformed percent-encoded byte sequences. Raw percent text and independently encoded round trips are
both exercised. Known regression inputs and successful typed queries seed every target; `rqs.dict` supplies syntax.

From the repository root:

```sh
make fuzz-setup
make fuzz-smoke
# Longer local exploration, keeping the input-size and per-input time bounds:
make fuzz-smoke FUZZ_SECONDS=3600 FUZZ_RUNS=-1
```

Tools are pinned to cargo-fuzz 0.13.2, nightly-2026-09-24 with rust-src, and libfuzzer-sys 0.4.13. AddressSanitizer,
debug assertions, and overflow checks remain enabled. The default smoke run stops each target after 10,000 executions
or 15 seconds, uses seed 5394771, a 5-second per-input timeout, and a 1024 MiB RSS limit. Compilation is outside the fuzz
time budget; the CI smoke step also has a 10-minute wall-clock limit. These are bounded smoke checks, not a proof that
all possible inputs are safe.

Committed seeds live in `fuzz/seeds/`. Mutable coverage corpora, artifacts, and build output are ignored. Initial seeds
include previously fixed parser regressions, mixed list/scalar/null queries, regex, floats, and temporal/UUID fields.
The property fixture replays known regressions with explicit expected outcomes even when fuzzing tools are unavailable.

Reproduce and minimize a failing input using the target printed in the log:

```sh
cargo +nightly-2026-09-24 fuzz run parsing fuzz/artifacts/parsing/crash-<hash>
cargo +nightly-2026-09-24 fuzz tmin parsing fuzz/artifacts/parsing/crash-<hash>
```

Copy the minimized query into `integration-tests/robustness/corpus/`, add one assertion proving the correct behavior,
and promote it to `fuzz/seeds/<target>/`. Commit proptest failure seeds too. Stable CI runs all targets inside the
required Rust check and uploads failing artifacts, mutable corpora, and property regression seeds for 14 days.

libfuzzer-sys is licensed `(MIT OR Apache-2.0) AND NCSA` and maintained by the Rust fuzzing project. It supplies the
instrumented libFuzzer runtime; cargo-fuzz supplies the build/run tooling. These dependencies remain outside the core
crate and ordinary tests. Maintenance costs are pinned nightly/tool upgrades, isolated compilation, triaging crashes,
and retaining minimized regressions. The fuzz lockfile is committed, audited in CI, and checked by Dependabot weekly.

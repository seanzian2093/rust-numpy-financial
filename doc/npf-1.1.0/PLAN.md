# NumPy Financial 1.1.0 Compatibility and Differential Testing Plan

## Objective

I will assess `rfinancial` against `numpy_financial` 1.1.0 in both directions:

- **Upstream Compatibility Suite** (`upstream_compatibility`): I will adapt applicable upstream Python test cases into Rust and compare against their published expectations.
- **Independent Differential Suite** (`independent_differential`): I will translate independent Rust cases into Python and compare both implementations against independently established expectations.

I will treat upstream results as a compatibility reference, not as infallible mathematical truth. I will preserve intentional Rust API differences and investigate disagreements before changing either expectations or algorithms.

## 1. Establish the baseline

1. I will review the current worktree and record the Rust commit and any local changes before starting.
2. I will record the Python, NumPy, `numpy_financial`, and Rust toolchain versions used for each comparison.
3. I will verify that the reference interpreter imports `numpy_financial` 1.1.0 from the intended virtual environment.
4. I will run `cargo test` and record existing failures, warnings, and ignored tests separately from new findings.
5. I will keep the current `1.4.1` patch-release scope separate from any future public API additions that may warrant a minor release.

## 2. Inventory and classify the cases

1. I will review the [1.1.0 release notes](https://github.com/numpy/numpy-financial/releases/tag/v1.1.0) and the tagged upstream implementation rather than relying on individual historical pull-request titles.
2. I will inventory the upstream tests in `numpy_financial/tests/test_financial.py` and the existing Rust unit tests, examples, and Python probes.
3. I will cover FV, PV, PMT, NPER, IPMT, PPMT, RATE, IRR, and MIRR.
4. I will identify existing upstream-derived Rust cases to avoid duplicating them.
5. I will classify each case as directly portable, portable with an explicit API mapping, or unsupported by the current Rust API.
6. I will keep Decimal arithmetic, fractional or negative periods that cannot fit `u32`, Python-specific argument handling, and broadcasting semantics separate from scalar compatibility.
7. I will extract useful scalar cases from array tests where possible, without claiming that scalar coverage validates broadcasting.

## 3. Define comparison rules

1. I will retain each upstream case's expected value and explicit tolerances, including effective defaults where the test omits them.
2. I will compare finite values using NumPy-compatible semantics:

   ```text
   abs(actual - expected) <= atol + rtol * abs(expected)
   ```

3. I will handle NaN, signed infinity, `None`, and errors explicitly rather than passing them through a finite-number tolerance check.
4. I will document approved mappings between Python results and Rust's `Result` and `Option` types. I will not automatically treat every NaN, exception, `None`, or error as equivalent.
5. I will record stable case identifiers, source test names, inputs, reference version, expected outcome, and tolerances.
6. I will preserve source attribution and applicable upstream license notices for adapted tests.

## 4. Build the Upstream Compatibility Suite

1. I will add a focused Rust integration-test entry point under `tests/upstream_compatibility.rs`, reusing suitable existing tests where practical.
2. I will begin with deterministic scalar examples and upstream regression cases.
3. I will prioritize zero-rate branches, payment timing, invalid payment periods, no-solution cases, RATE convergence, and IRR edge cases.
4. I will include the upstream NPV case for a discount rate of `-1`.
5. I will verify IRR multiple-root selection against the actual 1.1.0 implementation, including mixed-sign candidate rates.
6. I will keep Rust-only constructor and builder validation tests alongside their existing coverage rather than duplicating every numerical case across every entry point.
7. I will run `cargo test --test upstream_compatibility` before making algorithm changes and classify every failure.
8. I will keep this suite independent of Python at Rust test runtime by checking in reviewed expectations.

## 5. Build the Independent Differential Suite

1. I will identify cases that originated independently of upstream expectations, including known-rate annuities and robustness cases.
2. I will add a Python runner under `nfinancial/` for the `independent_differential` suite and a corresponding Rust comparison path.
3. I will preserve each case's original expectation and mathematical rationale separately from the observed Python and Rust outputs.
4. I will capture Python warnings and exception details instead of suppressing warnings or converting all exceptions into NaN.
5. I will record Rust outcomes as finite values, non-finite values, `None`, or errors without collapsing distinct outcomes.
6. I will reuse one deterministic case definition where practical to prevent the Python and Rust inputs from drifting.
7. I will run the same cases through `numpy_financial` 1.1.0 and the current Rust implementation, retaining enough input precision to reproduce disagreements.

## 6. Investigate disagreements independently

1. I will classify discrepancies as Rust defects, suspected upstream defects, intentional API differences, root-selection policy differences, or numerical conditioning issues.
2. I will reduce each unexplained discrepancy to a small reproducible case.
3. I will use analytical solutions, financial identities, or higher-precision calculations where necessary to establish an independent expectation.
4. For IRR, I will distinguish a valid but differently selected root from an invalid root or a missed solution.
5. I will check scale-aware NPV residuals at returned IRRs and retain known-rate annuities as independent checks.
6. I will examine search-domain boundaries and closely spaced or repeated roots separately from ordinary convergence cases.
7. I will not overwrite Rust expectations merely because Python returns a different result.
8. I will prepare minimal upstream reproducers for confirmed suspected defects, including versions, actual behavior, expected behavior, and supporting calculations. I will leave issue submission as a separate action.

## 7. Make and validate targeted fixes

1. I will add a focused failing regression test before correcting each confirmed Rust defect.
2. I will make the smallest implementation change that addresses the cause and preserve existing APIs unless an intentional change is approved.
3. I will run the affected tests first, then both compatibility suites and `cargo test`.
4. I will use existing formatting and lint tooling to check changed Rust code without reformatting unrelated files.
5. I will run relevant Criterion benchmarks if solver or numerical-kernel changes could affect performance.
6. I will keep proposed batching, broadcasting, or configurable root-selection APIs outside the patch-fix scope.

## 8. Refresh the IRR comparison documentation

1. I will update the comparison section in [IRR_SOLVER.md](../../IRR_SOLVER.md) only after running a reproducible Rust-versus-Python comparison.
2. I will record the reference versions, Rust revision, seed, case count, tolerance rules, and comparison command.
3. I will use identical inputs when attributing differences to a reference-version change; I will label a changed seed as a different sample.
4. I will define mutually exclusive result categories and verify that per-family counts sum to their totals.
5. I will distinguish Python-only results, Rust-only results, differing finite values, and execution errors.
6. I will retain historical 1.0.0 results as explicitly historical or replace them with clearly labeled 1.1.0 results, without mixing the two.
7. I will revise surrounding explanations and solver documentation to match verified current behavior.

## 9. Complete the review and checkpoint the work

1. I will summarize which upstream cases were ported, which independent cases were compared, and which features remain unsupported.
2. I will report confirmed defects, intentional differences, unresolved discrepancies, and validation results separately.
3. I will review the final diff and keep test infrastructure, behavioral fixes, and documentation changes in coherent commits.
4. I will confirm patch-release suitability for `1.4.1`; tests and documentation alone will not justify a `1.5.0` release.
5. I will keep committing, pushing, publishing, and submitting upstream issues as separate actions rather than automatic consequences of running the suites.

## Completion criteria

- I can run the upstream compatibility tests through Cargo without a Python runtime dependency.
- I can reproduce independent differential comparisons with recorded versions and deterministic inputs.
- I have accounted for every included case as a pass, an explicit intentional difference, an unsupported feature, or an actionable discrepancy.
- I have preserved independent expectations rather than replacing them with observed Python outputs.
- I have regenerated any published comparison table from measured results and documented its methodology.
- I have validated all implemented fixes and documented remaining limitations.

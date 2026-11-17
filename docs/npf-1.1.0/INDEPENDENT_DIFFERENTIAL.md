# Independent Differential Suite

## Purpose and scope

This suite is to compare Rust and `numpy_financial` 1.1.0 against expectations that do not come
from observed NumPy results. This suite is kept separate from the
[Upstream Compatibility Suite](../../tests/upstream_compatibility.rs).

A [fixture corpus](../../tests/fixtures/independent_differential.json) is maintained
with 36 deterministic cases across all ten public financial functions:

- Retain known-rate annuities of lengths 3, 4, 7, 15, 16, 17, 64, 257, and
  1024, plus quadratic and loss-making IRR cases.
- Retain analytic IRR roots outside the former scan bounds, zero-padded
  cash flows, and sign-reversed cash flows.
- Distinguish independently factored multiple-root polynomials from the
  documented compatibility policy that selects among their roots.
- Retain zero-interest identities, validation cases, and extreme-value
  diagnostics from the existing Rust tests and robustness example.
- Add three explicitly labeled analytic extensions: zero-interest PMT,
  zero-interest PPMT, and two-period compound-growth RATE.

Record the source and rationale for every case. Overflow expectations describe
the repository's existing floating-point behavior, not a mathematically finite
answer. This is not to duplicate NumPy-derived numerical references, Rust-only map/builder
validation, private solver-helper tests, or the upstream Python suite.

## Fixture creation and maintenance

Developers are expected to manually curate the [fixture corpus](../../tests/fixtures/independent_differential.json);
no script generates this file. Cases are selected from existing Rust tests and
robustness examples, and add explicitly labeled analytical cases. For each
entry, record its source, rationale, inputs, independent expectation, and
tolerances. Where necessary, separately document an approved Python outcome
mapping; Do not populate independent numerical expectations from NumPy output.

Only annuity cash-flow arrays are generated at runtime. Each annuity is stored
as an `irr_annuity` recipe containing `length`, `principal`, and `rate`. The
[Rust resolver](../../tests/independent_differential.rs) expands it using:

```rust
n = length - 1
payment = principal * rate / (1 - (1 + rate)^(-n))
cash_flows = [-principal, payment repeated n times]
```

The exact expanded array is passed to Python through the Rust report, so both
implementations receive identical inputs. This expansion does not modify the
fixture file.

Add or revise fixture entries deliberately, review their provenance and
mathematical or behavioral justification, then run both suites using the
commands below. Neither suite regenerates or overwrites the fixture, and a
disagreement is not a reason to automatically replace its expectation with
either implementation's observed result.

### Adding a case

Treat the fixture as reviewed test source code, not generated output.

1. Identify a meaningful coverage gap: a regression, boundary condition,
   financial identity, or analytically known answer.
2. Add an entry with a unique, descriptive, stable `id`; a `source`
   identifying its origin; a `rationale` justifying the expectation; and
   explicit inputs, expected outcome, `rtol`, and `atol`.
3. Establish the expectation independently rather than copying the current
   NumPy result. I label compatibility-policy expectations explicitly.
4. For a production bug, add a focused Rust unit test.
5. Use existing input formats where possible. An additional case using a
   supported format needs only a JSON entry; a new format also requires
   corresponding Rust and Python runner support.

### Modifying a case

1. Keep the ID when correcting or refining the same scenario.
2. Explain changes to inputs, expectations, or tolerances in the commit and
   keep the case's rationale accurate.
3. Investigate disagreements before changing expectations. I never loosen a
   tolerance merely to make a failure disappear.
4. If the scenario changes substantially, add a new case rather than
   repurposing the existing ID.

### Removing a case

Remove a case only when it is demonstrably redundant, invalid, or outside
the supported scope. Record the reason and any replacement coverage in the
commit. A passing result or an inconvenient failure is not sufficient reason
to remove a case.

### Validating and recording fixture changes

1. Run the Rust independent expectations, the Python differential runner,
   and the Python comparison-tool unit tests using the commands below.
2. Review the generated report for expected classifications, warning
   details, and case counts. I account for every added, removed, or modified
   case rather than checking only the overall exit status.
3. Update this document's case count and scope when necessary.
4. Commit fixture changes with directly related tests, implementation
   changes, and documentation. I keep generated reports untracked.

## Commands

From the repository root, check the independent Rust expectations without
requiring Python:

```sh
cargo test --test independent_differential
```

The Rust-only report by default is saved to`target/independent_differential_rust.json`,
relative to the crate root, regardless of the current working directory. Each run
overwrites this file. Keep it separate from the combined Python comparison report and
under the ignored `target/` directory. Override the Rust report path:

```sh
RFINANCIAL_DIFFERENTIAL_REPORT=/tmp/independent-rust.json cargo test --test independent_differential
```

Missing report directories are created automatically. Relative override paths
resolve from the test process's working directory. The Python runner continues
to use this override for its temporary Rust report.

Compare both runtimes using the existing virtual environment, which must
contain NumPy and exactly `numpy_financial` 1.1.0:

```sh
.venv/bin/python nfinancial/independent_differential.py
```

Find the resulting report at
`target/independent_differential.json`, or choose an explicit location:

```sh
.venv/bin/python nfinancial/independent_differential.py --output /tmp/independent-differential.json
```

Test the comparison/reporting logic separately:

```sh
.venv/bin/python -m unittest discover -s nfinancial -p 'test_independent_differential.py'
```

## Tests of the comparison tool

Use [test_independent_differential.py](../../nfinancial/test_independent_differential.py)
to test the comparison and reporting infrastructure, not the financial implementations.
Its seven unit tests verify:

1. It accepts results at the numerical tolerance boundary, reject results beyond
   it, and reject matching Rust/Python answers when both violate the independent
   expectation.
2. It treats NaN and `None` as distinct unless the case explicitly supplies an
   approved outcome mapping.
3. It preserves warning categories and messages, and exception classes and
   messages.
4. It serializes nonfinite outcomes as valid JSON and distinguish positive from
   negative infinity.
5. It calculates scaled IRR residuals for very large coefficients and identify
   different valid roots.
6. It writes a failure report and return a nonzero exit status for a numerical
   expectation failure.
7. It exposes a failed Cargo execution and return a nonzero exit status even
   when its reported numerical results match expectations.

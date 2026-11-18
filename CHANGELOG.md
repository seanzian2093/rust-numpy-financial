# Changelog

## 1.4.1

### Added

- Independent Differential Suite with shared deterministic cases, preserved
  mathematical and Rust-origin expectations, and a NumPy Financial 1.1.0
  comparison runner that records warnings, errors, versions, and explicit
  API differences. This is to record cases that originates from Rust
  implementation but produces different output from `numpy_financial`.

### Fixed

- Three Independent Differential Suite discrepancies now match
  `numpy_financial` 1.1.0:
  - NPER returns `Ok(Some(NaN))` instead of `Ok(None)` for rates at or below `-1`;
  - IRR returns `Ok(Some(NaN))` instead of an error for one-element cash flows;
  - and NPV returns `Ok(0.0)` instead of an error for empty cash flows, including
  at a rate of `-1`.
  These changes also apply to the corresponding struct and map APIs. Empty IRR
  inputs and multi-element same-sign IRR validation are unchanged.
- IRR now matches the `numpy_financial` 1.1.0 default selection rule among
  detected roots: prefer nonnegative rates when available, then choose the
  rate closest to zero.
  - This fixes `test_gh_39` and `test_gh_44`, where the previous rule selecte
  a valid negative IRR instead of the expected positive IRR. Multiple-root search
  bounds are unchanged.
- IRR now uses adaptive bracketing for cash flows with exactly one sign
  change, allowing roots outside the previous `1 + rate` search interval
  of `1e-6..=1e3`.
  - This addresses the `numpy_financial` 1.1.0 `test_gh_15` regression near a rate
  of `-1.0`. Cash flows with multiple sign changes still use the bounded search.
- NPV now returns `Ok(NaN)` for non-empty cash flows when the discount rate
  is exactly `-1.0`, matching `numpy_financial` 1.1.0. This also applies to
  the struct and map-based NPV APIs. Empty cash flows now return zero as
  documented above; non-empty calculations at other rates are unchanged.

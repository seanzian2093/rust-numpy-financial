# Changelog

## Unreleased

### Fixed

- IRR now matches the `numpy_financial` 1.1.0 default selection rule among
  detected roots: prefer nonnegative rates when available, then choose the
  rate closest to zero. This fixes `test_gh_39` and `test_gh_44`, where the
  previous rule selected a valid negative IRR instead of the expected
  positive IRR. Multiple-root search bounds are unchanged.
- IRR now uses adaptive bracketing for cash flows with exactly one sign
  change, allowing roots outside the previous `1 + rate` search interval
  of `1e-6..=1e3`. This addresses the `numpy_financial` 1.1.0 `test_gh_15`
  regression near a rate of `-1.0`. Cash flows with multiple sign changes
  still use the bounded search.
- NPV now returns `Ok(NaN)` for non-empty cash flows when the discount rate
  is exactly `-1.0`, matching `numpy_financial` 1.1.0. This also applies to
  the struct and map-based NPV APIs. Empty cash flows still return an error;
  calculations at other rates are unchanged.

//! Deterministic float cases adapted from numpy-financial 1.1.0:
//! https://github.com/numpy/numpy-financial/blob/v1.1.0/numpy_financial/tests/test_financial.py
//!
//! Array cases are evaluated as individual scalar calls, not as broadcasting tests.
//! Decimal cases, Python argument aliases/defaults, ndarray shape/type assertions,
//! mismatched rate-array lengths, and Hypothesis smoke tests are not ported:
//! the Rust API has no corresponding runtime features.
//!
//! Explicit outcome mappings: RATE non-convergence and IPMT/PPMT period zero map
//! Python NaN to Rust `Ok(None)`. Same-sign IRR/MIRR cases map both Python's
//! default NaN and opt-in NoRealSolutionError to Rust `Err(Error::ParaError(_))`.
//! There is no separate Rust `raise_exceptions` mode. Other outcomes remain strict.
//!
//! Run with `cargo test --test upstream_compatibility`. Compatibility failures
//! are deliberately not ignored or converted into expected failures.

/*
Copyright (c) 2005-2019, NumPy Developers.
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

    * Redistributions of source code must retain the above copyright
       notice, this list of conditions and the following disclaimer.

    * Redistributions in binary form must reproduce the above
       copyright notice, this list of conditions and the following
       disclaimer in the documentation and/or other materials provided
       with the distribution.

    * Neither the name of the NumPy Developers nor the names of any
       contributors may be used to endorse or promote products derived
       from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

#![allow(clippy::excessive_precision)]
use rfinancial::{Error, WhenType};

const RTOL: f64 = 1e-7;
const ATOL: f64 = 0.0;

#[track_caller]
fn assert_allclose(actual: f64, expected: f64, rtol: f64, atol: f64) {
    let close = if expected.is_nan() {
        actual.is_nan()
    } else if expected.is_infinite() {
        actual == expected
    } else {
        actual.is_finite() && (actual - expected).abs() <= atol + rtol * expected.abs()
    };
    assert!(
        close,
        "actual={actual:?}, expected={expected:?}, rtol={rtol}, atol={atol}"
    );
}

mod pv {
    use super::*;

    #[test]
    fn test_pv() {
        assert_allclose(
            rfinancial::pv(0.07, 20, 12000.0, 0.0, WhenType::End).unwrap(),
            -127128.17,
            1e-2,
            ATOL,
        );
    }
}

mod rate {
    use super::*;

    fn calculate(nper: u32, pmt: f64, pv: f64, fv: f64, when: WhenType) -> Option<f64> {
        rfinancial::rate(nper, pmt, pv, fv, when, 0.1, 1e-6, 100).unwrap()
    }

    #[test]
    fn test_rate() {
        assert_allclose(
            calculate(10, 0.0, -3500.0, 10000.0, WhenType::End).unwrap(),
            0.1107,
            1e-4,
            ATOL,
        );
    }

    #[test]
    fn test_rate_with_infeasible_solution() {
        for when in [WhenType::End, WhenType::Begin] {
            assert_eq!(calculate(12, 400.0, 10000.0, 5000.0, when), None);
        }
    }

    #[test]
    fn test_gh48_scalar_cases() {
        for (pv, fv, expected) in [
            (-593.06, 214.07, Some(-0.39920185)),
            (-4725.38, 4509.97, Some(-0.02305873)),
            (-662.05, 224.11, Some(-0.41818459)),
            (-428.78, 686.29, Some(0.26513414)),
            (-13.65, -329.67, None),
        ] {
            let actual = calculate(2, 0.0, pv, fv, WhenType::End);
            match expected {
                Some(expected) => assert_allclose(actual.unwrap(), expected, RTOL, ATOL),
                None => assert_eq!(actual, None),
            }
        }
    }
}

mod npv {
    use super::*;

    #[test]
    fn test_npv() {
        assert_allclose(
            rfinancial::npv(&[-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0], 0.05).unwrap(),
            122.89,
            1e-2,
            ATOL,
        );
    }

    #[test]
    fn test_rate_of_negative_one_returns_nan() {
        assert_allclose(
            rfinancial::npv(&[0.0, 1.0, 2.0, 3.0, 4.0], -1.0).unwrap(),
            f64::NAN,
            RTOL,
            ATOL,
        );
    }
}

mod pmt {
    use super::*;

    #[test]
    fn test_pmt_simple() {
        assert_allclose(
            rfinancial::pmt(0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End).unwrap(),
            -304.145914,
            RTOL,
            ATOL,
        );
    }

    #[test]
    fn test_pmt_zero_rate() {
        assert_allclose(
            rfinancial::pmt(0.0, 60, 15000.0, 0.0, WhenType::End).unwrap(),
            -250.0,
            RTOL,
            ATOL,
        );
    }

    #[test]
    fn test_pmt_broadcast_scalar_cases() {
        for (rate, nper, pv, expected) in [
            (0.0, 12, 2000.0, -166.66667),
            (0.8, 3, 20000.0, -19311.258),
            (0.3, 12, 2000.0, -626.90814),
            (0.8, 3, 20000.0, -19311.258),
        ] {
            assert_allclose(
                rfinancial::pmt(rate, nper, pv, 0.0, WhenType::End).unwrap(),
                expected,
                RTOL,
                ATOL,
            );
        }
    }
}

mod mirr {
    use super::*;

    #[test]
    fn test_mirr() {
        let cases: &[(&[f64], f64, f64, f64, f64)] = &[
            (
                &[
                    -4500.0, -800.0, 800.0, 800.0, 600.0, 600.0, 800.0, 800.0, 700.0, 3000.0,
                ],
                0.08,
                0.055,
                0.0666,
                1e-4,
            ),
            (
                &[-120000.0, 39000.0, 30000.0, 21000.0, 37000.0, 46000.0],
                0.10,
                0.12,
                0.126094,
                1e-6,
            ),
            (
                &[100.0, 200.0, -50.0, 300.0, -200.0],
                0.05,
                0.06,
                0.3428,
                1e-4,
            ),
        ];
        for &(values, finance_rate, reinvest_rate, expected, atol) in cases {
            assert_allclose(
                rfinancial::mirr(values, finance_rate, reinvest_rate)
                    .unwrap()
                    .unwrap(),
                expected,
                RTOL,
                atol,
            );
        }
    }

    #[test]
    fn test_mirr_broadcast_scalar_cases() {
        let values = [
            [-4500.0, -800.0, 800.0, 800.0, 600.0],
            [-120000.0, 39000.0, 30000.0, 21000.0, 37000.0],
            [100.0, 200.0, -50.0, 300.0, -200.0],
        ];
        let rates = [(0.05, 0.08), (0.08, 0.10), (0.10, 0.12)];
        let expected = [
            [-0.1784449, -0.17328716, -0.1684366],
            [0.04627293, 0.05437856, 0.06252201],
            [0.35712458, 0.40628857, 0.44435295],
        ];
        for (values, expected) in values.iter().zip(expected) {
            for ((finance_rate, reinvest_rate), expected) in rates.iter().zip(expected) {
                assert_allclose(
                    rfinancial::mirr(values, *finance_rate, *reinvest_rate)
                        .unwrap()
                        .unwrap(),
                    expected,
                    RTOL,
                    ATOL,
                );
            }
        }
    }

    #[test]
    fn test_mirr_no_real_solution() {
        assert!(matches!(
            rfinancial::mirr(&[39000.0, 30000.0, 21000.0, 37000.0, 46000.0], 0.10, 0.12),
            Err(Error::ParaError(_))
        ));
    }
}

mod nper {
    use super::*;

    #[test]
    fn test_basic_values() {
        for (rate, expected) in [(0.0, 50.0), (0.075, 21.544944)] {
            assert_allclose(
                rfinancial::nper(rate, -2000.0, 0.0, 100000.0, WhenType::End)
                    .unwrap()
                    .unwrap(),
                expected,
                1e-5,
                ATOL,
            );
        }
    }

    #[test]
    fn test_gh_18() {
        assert_allclose(
            rfinancial::nper(0.1, 0.0, -500.0, 1500.0, WhenType::End)
                .unwrap()
                .unwrap(),
            11.52670461,
            RTOL,
            ATOL,
        );
    }

    #[test]
    fn test_infinite_payments() {
        assert_allclose(
            rfinancial::nper(0.0, -0.0, 1000.0, 0.0, WhenType::End)
                .unwrap()
                .unwrap(),
            f64::INFINITY,
            RTOL,
            ATOL,
        );
    }

    #[test]
    fn test_no_interest() {
        assert_eq!(
            rfinancial::nper(0.0, -100.0, 1000.0, 0.0, WhenType::End).unwrap(),
            Some(10.0)
        );
    }

    #[test]
    fn test_broadcast_scalar_cases() {
        for (when, expected) in [(WhenType::End, 21.5449442), (WhenType::Begin, 20.76156441)] {
            // Upstream passes 4 as the positional rtol, not as decimal places.
            assert_allclose(
                rfinancial::nper(0.075, -2000.0, 0.0, 100000.0, when)
                    .unwrap()
                    .unwrap(),
                expected,
                4.0,
                ATOL,
            );
        }
    }
}

mod ppmt {
    use super::*;

    #[test]
    fn test_float() {
        assert_allclose(
            rfinancial::ppmt(0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::End)
                .unwrap()
                .unwrap(),
            -710.25,
            1e-4,
            ATOL,
        );
    }

    #[test]
    fn test_when_is_begin() {
        assert_allclose(
            rfinancial::ppmt(0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::Begin)
                .unwrap()
                .unwrap(),
            -1158.929712,
            1e-9,
            ATOL,
        );
    }

    #[test]
    fn test_when_is_end() {
        assert_allclose(
            rfinancial::ppmt(0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::End)
                .unwrap()
                .unwrap(),
            -710.254126,
            1e-9,
            ATOL,
        );
    }

    #[test]
    fn test_invalid_per() {
        assert_eq!(
            rfinancial::ppmt(0.1 / 12.0, 0, 60, 15000.0, 0.0, WhenType::End).unwrap(),
            None
        );
    }

    #[test]
    fn test_broadcast_scalar_cases() {
        let expected = [
            (
                WhenType::End,
                [-75.62318601, -76.25337923, -76.88882405, -77.52956425],
            ),
            (
                WhenType::Begin,
                [-75.62318601, -75.62318601, -76.88882405, -76.88882405],
            ),
        ];
        for (style, values) in expected {
            for (per, expected) in (1..=4).zip(values) {
                let when = if style == WhenType::Begin && per % 2 == 0 {
                    WhenType::Begin
                } else {
                    WhenType::End
                };
                assert_allclose(
                    rfinancial::ppmt(0.1 / 12.0, per, 24, 2000.0, 0.0, when)
                        .unwrap()
                        .unwrap(),
                    expected,
                    1e-5,
                    ATOL,
                );
            }
        }
    }
}

mod ipmt {
    use super::*;

    #[test]
    fn test_float_and_when_is_end() {
        assert_allclose(
            rfinancial::ipmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End)
                .unwrap()
                .unwrap(),
            -16.666667,
            1e-6,
            ATOL,
        );
    }

    #[test]
    fn test_when_is_begin() {
        assert_eq!(
            rfinancial::ipmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::Begin).unwrap(),
            Some(0.0)
        );
    }

    #[test]
    fn test_gh_17() {
        for (per, expected) in [
            (0, None),
            (1, Some(0.0)),
            (2, Some(-594.107158)),
            (3, Some(-592.971592)),
        ] {
            let actual = rfinancial::ipmt(
                0.001988079518355057,
                per,
                360,
                300000.0,
                0.0,
                WhenType::Begin,
            )
            .unwrap();
            match expected {
                Some(expected) => assert_allclose(actual.unwrap(), expected, 1e-6, ATOL),
                None => assert_eq!(actual, None),
            }
        }
    }

    #[test]
    fn test_broadcasting_scalar_cases() {
        let expected = [
            None,
            Some(-16.66666667),
            Some(-16.03647345),
            Some(-15.40102862),
            Some(-14.76028842),
        ];
        for (per, expected) in (0..=4).zip(expected) {
            let actual = rfinancial::ipmt(0.1 / 12.0, per, 24, 2000.0, 0.0, WhenType::End).unwrap();
            match expected {
                Some(expected) => assert_allclose(actual.unwrap(), expected, 1e-6, ATOL),
                None => assert_eq!(actual, None),
            }
        }
    }
}

mod fv {
    use super::*;

    #[test]
    fn test_float() {
        assert_allclose(
            rfinancial::fv(0.075, 20, -2000.0, 0.0, WhenType::End).unwrap(),
            86609.362673042924,
            1e-10,
            ATOL,
        );
    }

    #[test]
    fn test_when_is_begin_float() {
        assert_allclose(
            rfinancial::fv(0.075, 20, -2000.0, 0.0, WhenType::Begin).unwrap(),
            93105.064874,
            1e-10,
            ATOL,
        );
    }

    #[test]
    fn test_when_is_end_float() {
        assert_allclose(
            rfinancial::fv(0.075, 20, -2000.0, 0.0, WhenType::End).unwrap(),
            86609.362673,
            1e-10,
            ATOL,
        );
    }

    #[test]
    fn test_broadcast_scalar_cases() {
        for (rate, expected) in [
            (0.1, [-610.510000, -671.561000]),
            (0.2, [-744.160000, -892.992000]),
        ] {
            for (when, expected) in [WhenType::End, WhenType::Begin].into_iter().zip(expected) {
                assert_allclose(
                    rfinancial::fv(rate, 5, 100.0, 0.0, when).unwrap(),
                    expected,
                    1e-10,
                    ATOL,
                );
            }
        }
    }

    #[test]
    fn test_some_rates_zero_scalar_cases() {
        for (rate, expected) in [(0.0, -500.0), (0.1, -610.51)] {
            assert_allclose(
                rfinancial::fv(rate, 5, 100.0, 0.0, WhenType::End).unwrap(),
                expected,
                1e-10,
                ATOL,
            );
        }
    }
}

mod irr {
    use super::*;

    #[test]
    fn test_npv_irr_congruence() {
        let values = [-40000.0, 5000.0, 8000.0, 12000.0, 30000.0];
        let rate = rfinancial::irr(&values).unwrap().unwrap();
        assert_allclose(rfinancial::npv(&values, rate).unwrap(), 0.0, 0.0, 1e-9);
    }

    #[test]
    fn test_basic_values() {
        let cases: &[(&[f64], f64)] = &[
            (
                &[-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0],
                0.0524,
            ),
            (&[-100.0, 0.0, 0.0, 74.0], -0.0955),
            (&[-100.0, 39.0, 59.0, 55.0, 20.0], 0.28095),
            (&[-100.0, 100.0, 0.0, -7.0], -0.0833),
            (&[-100.0, 100.0, 0.0, 7.0], 0.06206),
            (&[-5.0, 10.5, 1.0, -8.0, 1.0], 0.0886),
        ];
        for &(values, expected) in cases {
            assert_allclose(
                rfinancial::irr(values).unwrap().unwrap(),
                expected,
                1e-2,
                ATOL,
            );
        }
    }

    #[test]
    fn test_trailing_zeros() {
        assert_allclose(
            rfinancial::irr(&[-5.0, 10.5, 1.0, -8.0, 1.0, 0.0, 0.0, 0.0])
                .unwrap()
                .unwrap(),
            0.0886,
            1e-2,
            ATOL,
        );
    }

    #[test]
    fn test_numpy_gh_6744() {
        for values in [[1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]] {
            assert!(matches!(rfinancial::irr(&values), Err(Error::ParaError(_))));
        }
    }

    #[test]
    fn test_gh_15() {
        let values = [
            -3000.0,
            2.3926932267015667e-07,
            4.1672087103345505e-16,
            5.3965110036378706e-25,
            5.1962551071806174e-34,
            3.7202955645436402e-43,
            1.9804961711632469e-52,
            7.8393517651814181e-62,
            2.3072565113911438e-71,
            5.0491839233308912e-81,
            8.2159177668499263e-91,
            9.9403244366963527e-101,
            8.942410813633967e-111,
            5.9816122646481191e-121,
            2.9750309031844241e-131,
            1.1002067043497954e-141,
            3.0252876563518021e-152,
            6.1854121948207909e-163,
            9.4032980015353301e-174,
            1.0629218520017728e-184,
            8.9337141847171845e-196,
            5.5830607698467935e-207,
            2.5943122036622652e-218,
            8.9635842466507006e-230,
            2.3027710094332358e-241,
            4.3987510596745562e-253,
            6.2476630372575209e-265,
            6.598046841695288e-277,
            5.1811095266842017e-289,
            3.0250999925830644e-301,
            1.3133070599585015e-313,
        ];
        let actual = rfinancial::irr(&values)
            .unwrap()
            .expect("upstream gh_15 expects a finite IRR");
        assert_allclose(actual, -0.9999999990596069, 1e-9, ATOL);
    }

    #[test]
    fn test_gh_39() {
        let values = [
            -217500.0,
            -217500.0,
            108466.80462450592,
            101129.96439328062,
            93793.12416205535,
            86456.28393083003,
            79119.44369960476,
            71782.60346837944,
            64445.76323715414,
            57108.92300592884,
            49772.08277470355,
            42435.24254347826,
            35098.40231225296,
            27761.56208102766,
            20424.721849802358,
            13087.88161857707,
            5751.041387351768,
            -1585.7988438735192,
            -8922.639075098821,
            -16259.479306324123,
            -23596.31953754941,
            -30933.159768774713,
            -38270.0,
            -45606.8402312253,
            -52943.680462450604,
            -60280.520693675906,
            -67617.36092490121,
        ];
        assert_allclose(rfinancial::irr(&values).unwrap().unwrap(), 0.12, RTOL, ATOL);
    }

    #[test]
    fn test_gh_44() {
        let values = [
            -1678.87, 771.96, 1814.05, 3520.30, 3552.95, 3584.99, 4789.91, -1.0,
        ];
        assert_allclose(
            rfinancial::irr(&values).unwrap().unwrap(),
            1.00426,
            1e-4,
            ATOL,
        );
    }

    #[test]
    fn test_irr_no_real_solution_exception() {
        assert!(matches!(
            rfinancial::irr(&[40000.0, 5000.0, 8000.0, 12000.0, 30000.0]),
            Err(Error::ParaError(_))
        ));
    }
}

//! Independent expectations, also exported for nfinancial/independent_differential.py.

use rfinancial::{Error, WhenType};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    rationale: String,
    input: Input,
    expected: Expected,
    python_expected: Option<Expected>,
    difference_reason: Option<String>,
    rtol: f64,
    atol: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "function", rename_all = "snake_case", deny_unknown_fields)]
enum Input {
    Fv {
        rate: f64,
        nper: u32,
        pmt: f64,
        pv: f64,
        when: Timing,
    },
    Pv {
        rate: f64,
        nper: u32,
        pmt: f64,
        fv: f64,
        when: Timing,
    },
    Pmt {
        rate: f64,
        nper: u32,
        pv: f64,
        fv: f64,
        when: Timing,
    },
    Nper {
        rate: f64,
        pmt: f64,
        pv: f64,
        fv: f64,
        when: Timing,
    },
    Ipmt {
        rate: f64,
        per: u32,
        nper: u32,
        pv: f64,
        fv: f64,
        when: Timing,
    },
    Ppmt {
        rate: f64,
        per: u32,
        nper: u32,
        pv: f64,
        fv: f64,
        when: Timing,
    },
    Rate {
        nper: u32,
        pmt: f64,
        pv: f64,
        fv: f64,
        when: Timing,
        guess: f64,
        tol: f64,
        maxiter: u32,
    },
    Irr {
        values: Vec<f64>,
    },
    IrrAnnuity {
        length: u32,
        principal: f64,
        rate: f64,
    },
    Npv {
        values: Vec<f64>,
        rate: f64,
    },
    Mirr {
        values: Vec<f64>,
        finance_rate: f64,
        reinvest_rate: f64,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Timing {
    End,
    Begin,
}

impl From<Timing> for WhenType {
    fn from(value: Timing) -> Self {
        match value {
            Timing::End => Self::End,
            Timing::Begin => Self::Begin,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Expected {
    Finite { value: f64 },
    Nan,
    PositiveInfinity,
    NegativeInfinity,
    None,
    Error { class: String },
    IrrRoot { residual_atol: f64 },
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    Finite {
        value: f64,
    },
    Nan,
    PositiveInfinity,
    NegativeInfinity,
    None,
    Error {
        class: &'static str,
        message: String,
    },
}

impl Outcome {
    fn number(value: f64) -> Self {
        if value.is_nan() {
            Self::Nan
        } else if value == f64::INFINITY {
            Self::PositiveInfinity
        } else if value == f64::NEG_INFINITY {
            Self::NegativeInfinity
        } else {
            Self::Finite { value }
        }
    }

    fn from_result(result: rfinancial::Result<Option<f64>>) -> Self {
        match result {
            Ok(Some(value)) => Self::number(value),
            Ok(None) => Self::None,
            Err(error) => {
                let class = match &error {
                    Error::ParaError(_) => "ParaError",
                    Error::ConstructorError(_) => "ConstructorError",
                    Error::OtherError(_) => "OtherError",
                };
                Self::Error {
                    class,
                    message: error.to_string(),
                }
            }
        }
    }
}

impl Input {
    fn resolve(self) -> Self {
        match self {
            Self::IrrAnnuity {
                length,
                principal,
                rate,
            } => {
                assert!(length >= 2 && principal > 0.0 && rate > 0.0);
                let n = f64::from(length - 1);
                let payment = principal * rate / (1.0 - (1.0 + rate).powf(-n));
                assert!(payment.is_finite());
                let mut values = vec![-principal];
                values.extend(std::iter::repeat_n(payment, (length - 1) as usize));
                Self::Irr { values }
            }
            other => other,
        }
    }

    fn evaluate(&self) -> Outcome {
        let result = match self {
            Self::Fv {
                rate,
                nper,
                pmt,
                pv,
                when,
            } => rfinancial::fv(*rate, *nper, *pmt, *pv, (*when).into()).map(Some),
            Self::Pv {
                rate,
                nper,
                pmt,
                fv,
                when,
            } => rfinancial::pv(*rate, *nper, *pmt, *fv, (*when).into()).map(Some),
            Self::Pmt {
                rate,
                nper,
                pv,
                fv,
                when,
            } => rfinancial::pmt(*rate, *nper, *pv, *fv, (*when).into()).map(Some),
            Self::Nper {
                rate,
                pmt,
                pv,
                fv,
                when,
            } => rfinancial::nper(*rate, *pmt, *pv, *fv, (*when).into()),
            Self::Ipmt {
                rate,
                per,
                nper,
                pv,
                fv,
                when,
            } => rfinancial::ipmt(*rate, *per, *nper, *pv, *fv, (*when).into()),
            Self::Ppmt {
                rate,
                per,
                nper,
                pv,
                fv,
                when,
            } => rfinancial::ppmt(*rate, *per, *nper, *pv, *fv, (*when).into()),
            Self::Rate {
                nper,
                pmt,
                pv,
                fv,
                when,
                guess,
                tol,
                maxiter,
            } => rfinancial::rate(
                *nper,
                *pmt,
                *pv,
                *fv,
                (*when).into(),
                *guess,
                *tol,
                *maxiter,
            ),
            Self::Irr { values } => rfinancial::irr(values),
            Self::Npv { values, rate } => rfinancial::npv(values, *rate).map(Some),
            Self::Mirr {
                values,
                finance_rate,
                reinvest_rate,
            } => rfinancial::mirr(values, *finance_rate, *reinvest_rate),
            Self::IrrAnnuity { .. } => {
                unreachable!("annuity must be materialized before evaluation")
            }
        };
        Outcome::from_result(result)
    }

    fn residual(&self, outcome: &Outcome) -> Option<f64> {
        let (Self::Irr { values }, Outcome::Finite { value: rate }) = (self, outcome) else {
            return None;
        };
        if *rate <= -1.0 {
            return None;
        }
        let coefficient_scale = values.iter().map(|v| v.abs()).fold(0.0, f64::max);
        if coefficient_scale == 0.0 {
            return None;
        }
        // Evaluate a scaled polynomial for x < 1 and scaled NPV otherwise.
        // Normalizing both coefficients and the residual avoids overflow.
        let x = 1.0 + rate;
        let (mut sum, mut scale) = (0.0, 0.0);
        if x < 1.0 {
            for value in values {
                let v = value / coefficient_scale;
                sum = sum * x + v;
                scale = scale * x + v.abs();
            }
        } else {
            for value in values.iter().rev() {
                let v = value / coefficient_scale;
                sum = sum / x + v;
                scale = scale / x + v.abs();
            }
        }
        (scale > 0.0).then(|| sum.abs() / scale)
    }
}

fn matches(case: &Case, outcome: &Outcome) -> bool {
    match (&case.expected, outcome) {
        (Expected::Finite { value: expected }, Outcome::Finite { value: actual }) => {
            (actual - expected).abs() <= case.atol + case.rtol * expected.abs()
        }
        (Expected::Nan, Outcome::Nan)
        | (Expected::PositiveInfinity, Outcome::PositiveInfinity)
        | (Expected::NegativeInfinity, Outcome::NegativeInfinity)
        | (Expected::None, Outcome::None) => true,
        (Expected::Error { class: expected }, Outcome::Error { class: actual, .. }) => {
            expected == actual
        }
        (Expected::IrrRoot { residual_atol }, _) => case
            .input
            .residual(outcome)
            .is_some_and(|r| r <= *residual_atol),
        _ => false,
    }
}

#[derive(Debug, Serialize)]
struct CaseResult {
    case: Case,
    outcome: Outcome,
    normalized_irr_residual: Option<f64>,
    matches_expected: bool,
}

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    crate_version: &'static str,
    cases: Vec<CaseResult>,
}

#[test]
fn independent_expectations() {
    let definitions: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/independent_differential.json"))
            .expect("valid independent fixture definitions");
    assert!(!definitions.is_empty());
    let mut ids = HashSet::new();
    let mut results = Vec::new();
    for mut case in definitions {
        assert!(
            ids.insert(case.id.clone()),
            "duplicate case id: {}",
            case.id
        );
        assert!(!case.source.is_empty() && !case.rationale.is_empty());
        assert!(case.rtol.is_finite() && case.rtol >= 0.0);
        assert!(case.atol.is_finite() && case.atol >= 0.0);
        assert_eq!(
            case.python_expected.is_some(),
            case.difference_reason.is_some()
        );
        case.input = case.input.resolve();
        let outcome = case.input.evaluate();
        let normalized_irr_residual = case.input.residual(&outcome);
        let matches_expected = matches(&case, &outcome);
        results.push(CaseResult {
            case,
            outcome,
            normalized_irr_residual,
            matches_expected,
        });
    }
    let report = Report {
        schema_version: 1,
        crate_version: env!("CARGO_PKG_VERSION"),
        cases: results,
    };
    let path = std::env::var_os("RFINANCIAL_DIFFERENTIAL_REPORT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/independent_differential_rust.json")
        });
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).unwrap_or_else(|error| {
            panic!(
                "cannot create report directory {}: {error}",
                parent.display()
            )
        });
    }
    // Export before asserting so the Python runner can retain failed outcomes.
    fs::write(
        &path,
        serde_json::to_string_pretty(&report).expect("serializable report"),
    )
    .unwrap_or_else(|error| panic!("cannot write Rust report {}: {error}", path.display()));
    let failures: Vec<_> = report
        .cases
        .iter()
        .filter(|row| !row.matches_expected)
        .collect();
    assert!(
        failures.is_empty(),
        "independent expectation failures: {failures:#?}"
    );
}

"""Compare independent Rust expectations with numpy_financial 1.1.0."""

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import warnings

import numpy as np
import numpy_financial as npf


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/independent_differential.json"
REFERENCE_VERSION = "1.1.0"
PASS_CLASSIFICATIONS = {"agreement", "documented_difference"}


def number_outcome(value):
    if math.isnan(value):
        return {"kind": "nan"}
    if math.isinf(value):
        return {"kind": "positive_infinity" if value > 0 else "negative_infinity"}
    return {"kind": "finite", "value": value}


def normalized_irr_residual(inputs, outcome):
    if inputs["function"] != "irr" or outcome["kind"] != "finite":
        return None
    rate = outcome["value"]
    if rate <= -1:
        return None
    values = inputs["values"]
    coefficient_scale = max((abs(v) for v in values), default=0)
    if coefficient_scale == 0:
        return None
    x = 1 + rate
    total = scale = 0.0
    # Use a scaled polynomial below x=1 and scaled NPV above it.
    for value in values if x < 1 else reversed(values):
        v = value / coefficient_scale
        total = total * x + v if x < 1 else total / x + v
        scale = scale * x + abs(v) if x < 1 else scale / x + abs(v)
    return abs(total) / scale if scale > 0 else None


def matches_expected(outcome, expected, case):
    kind = expected["kind"]
    if kind == "irr_root":
        residual = normalized_irr_residual(case["input"], outcome)
        return residual is not None and residual <= expected["residual_atol"]
    if outcome["kind"] != kind:
        return False
    if kind == "finite":
        return (
            abs(outcome["value"] - expected["value"])
            <= case["atol"] + case["rtol"] * abs(expected["value"])
        )
    if kind == "error":
        return outcome["class"] == expected["class"]
    return kind in {"nan", "positive_infinity", "negative_infinity", "none"}


def evaluate_python(inputs):
    functions = {
        "fv": npf.fv, "pv": npf.pv, "pmt": npf.pmt, "nper": npf.nper,
        "ipmt": npf.ipmt, "ppmt": npf.ppmt, "rate": npf.rate,
        "irr": npf.irr, "npv": npf.npv, "mirr": npf.mirr,
    }
    arguments = {key: value for key, value in inputs.items() if key != "function"}
    function = functions[inputs["function"]]
    with warnings.catch_warnings(record=True) as captured:
        warnings.simplefilter("always")
        try:
            result = function(**arguments)
        except (ArithmeticError, ValueError, TypeError, IndexError, np.linalg.LinAlgError) as error:
            outcome = {
                "kind": "error",
                "class": type(error).__name__,
                "module": type(error).__module__,
                "message": str(error),
            }
        else:
            outcome = {"kind": "none"} if result is None else number_outcome(float(result))
    return {
        "outcome": outcome,
        "warnings": [
            {"category": item.category.__name__, "message": str(item.message)}
            for item in captured
        ],
        "normalized_irr_residual": normalized_irr_residual(inputs, outcome),
    }


def compare(row, python):
    case = row["case"]
    rust_ok = matches_expected(row["outcome"], case["expected"], case)
    expected_python = case["python_expected"] or case["expected"]
    python_ok = matches_expected(python["outcome"], expected_python, case)
    agreement = matches_expected(row["outcome"], python["outcome"], case)
    if rust_ok != row["matches_expected"]:
        raise ValueError(f"Inconsistent Rust expectation check: {case['id']}")
    if not rust_ok or not python_ok:
        classification = "unexpected_result"
    elif case["python_expected"] is not None:
        classification = "documented_difference"
    elif agreement:
        classification = "agreement"
    elif (
        row["normalized_irr_residual"] is not None
        and python["normalized_irr_residual"] is not None
        and max(row["normalized_irr_residual"], python["normalized_irr_residual"]) <= 1e-8
    ):
        classification = "different_valid_irr_roots"
    elif row["outcome"]["kind"] == python["outcome"]["kind"] == "finite":
        classification = "numerical_difference"
    else:
        classification = "outcome_difference"
    return {
        **row,
        "python": {**python, "matches_expected": python_ok},
        "agreement": agreement,
        "classification": classification,
    }


def command_output(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def run_rust():
    with tempfile.TemporaryDirectory(prefix="rfinancial-differential-") as directory:
        report_path = Path(directory) / "rust.json"
        environment = dict(os.environ, RFINANCIAL_DIFFERENTIAL_REPORT=str(report_path))
        completed = subprocess.run(
            [
                "cargo", "test", "--quiet", "--test", "independent_differential",
                "independent_expectations", "--", "--exact",
            ],
            cwd=ROOT, env=environment, text=True, capture_output=True, check=False,
        )
        if not report_path.is_file():
            raise RuntimeError(
                f"Cargo exited {completed.returncode} without a Rust report.\n"
                f"{completed.stdout}\n{completed.stderr}"
            )
        report = json.loads(report_path.read_text(encoding="utf-8"))
    if report["schema_version"] != 1 or not report["cases"]:
        raise ValueError("Unsupported or empty Rust differential report")
    ids = [row["case"]["id"] for row in report["cases"]]
    if len(set(ids)) != len(ids):
        raise ValueError("Duplicate case IDs in Rust differential report")
    return report, completed


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output", type=Path, default=ROOT / "target/independent_differential_report.json",
        help="JSON report path (default: target/independent_differential_report.json)",
    )
    args = parser.parse_args(argv)
    if npf.__version__ != REFERENCE_VERSION:
        parser.error(f"numpy_financial {REFERENCE_VERSION} required; found {npf.__version__}")

    rust_report, cargo = run_rust()
    print(f"Comparing {len(rust_report['cases'])} cases with numpy_financial {npf.__version__}...", flush=True)
    rows = [
        compare(row, evaluate_python(row["case"]["input"]))
        for row in rust_report["cases"]
    ]
    counts = Counter(row["classification"] for row in rows)
    failed = sum(count for key, count in counts.items() if key not in PASS_CLASSIFICATIONS)
    report = {
        "schema_version": 1,
        "suite": "independent_differential",
        "metadata": {
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
            "python": platform.python_version(),
            "numpy": np.__version__,
            "numpy_financial": npf.__version__,
            "rustc": command_output("rustc", "--version"),
            "cargo": command_output("cargo", "--version"),
            "crate_version": rust_report["crate_version"],
            "git_revision": command_output("git", "rev-parse", "HEAD"),
            "git_dirty": bool(command_output("git", "status", "--porcelain")),
            "fixture": str(FIXTURE.relative_to(ROOT)),
            "fixture_sha256": hashlib.sha256(FIXTURE.read_bytes()).hexdigest(),
            "generation": "deterministic; no random seed; Rust materializes shared annuities",
        },
        "rust_execution": {
            "exit_code": cargo.returncode, "stdout": cargo.stdout, "stderr": cargo.stderr,
        },
        "summary": {
            "total": len(rows), "passed": len(rows) - failed, "failed": failed,
            "classifications": dict(counts),
            "python_warnings": sum(len(row["python"]["warnings"]) for row in rows),
        },
        "cases": rows,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    if cargo.returncode:
        print(f"Rust validation failed (Cargo exit {cargo.returncode}).")
        print(cargo.stdout)
        print(cargo.stderr)
    for row in rows:
        if row["classification"] not in PASS_CLASSIFICATIONS:
            print(
                f"{row['case']['id']}: {row['classification']}; "
                f"Rust={row['outcome']}; Python={row['python']['outcome']}"
            )
    print(f"{len(rows) - failed} passed, {failed} failed; {dict(counts)}")
    print(f"Report: {args.output.resolve()}")
    return int(bool(failed or cargo.returncode))


if __name__ == "__main__":
    raise SystemExit(main())

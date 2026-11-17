from contextlib import redirect_stdout
from io import StringIO
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import warnings

import independent_differential as suite


class DifferentialTests(unittest.TestCase):
    def case(self):
        return {
            "id": "test", "input": {"function": "npv", "rate": 0, "values": [1]},
            "expected": {"kind": "finite", "value": 1},
            "python_expected": None, "difference_reason": None,
            "rtol": 0.125, "atol": 0,
        }

    def row(self, case, outcome):
        return {
            "case": case, "outcome": outcome, "normalized_irr_residual": None,
            "matches_expected": suite.matches_expected(outcome, case["expected"], case),
        }

    def test_tolerance_boundary_and_same_wrong_answer(self):
        case = self.case()
        self.assertTrue(suite.matches_expected({"kind": "finite", "value": 1.125}, case["expected"], case))
        self.assertFalse(suite.matches_expected({"kind": "finite", "value": 1.126}, case["expected"], case))
        wrong = {"kind": "finite", "value": 2}
        compared = suite.compare(self.row(case, wrong), {"outcome": wrong, "normalized_irr_residual": None})
        self.assertTrue(compared["agreement"])
        self.assertEqual(compared["classification"], "unexpected_result")

    def test_nan_none_mapping_is_case_specific(self):
        case = self.case()
        case["expected"] = {"kind": "none"}
        python = {"outcome": {"kind": "nan"}, "normalized_irr_residual": None}
        row = self.row(case, {"kind": "none"})
        self.assertEqual(suite.compare(row, python)["classification"], "unexpected_result")
        case["python_expected"] = {"kind": "nan"}
        case["difference_reason"] = "Explicit test-only mapping."
        self.assertEqual(suite.compare(row, python)["classification"], "documented_difference")

    def test_python_warnings_and_exception_are_preserved(self):
        def fail(**_):
            warnings.warn("diagnostic warning", RuntimeWarning)
            raise ValueError("diagnostic failure")

        with patch.object(suite.npf, "npv", side_effect=fail):
            result = suite.evaluate_python(self.case()["input"])
        self.assertEqual(result["outcome"]["class"], "ValueError")
        self.assertEqual(result["outcome"]["message"], "diagnostic failure")
        self.assertEqual(result["warnings"], [{"category": "RuntimeWarning", "message": "diagnostic warning"}])

    def test_nonfinite_json_and_infinity_sign(self):
        outcomes = [suite.number_outcome(value) for value in (float("nan"), float("inf"), -float("inf"))]
        json.dumps(outcomes, allow_nan=False)
        self.assertFalse(suite.matches_expected(outcomes[1], outcomes[2], self.case()))
        self.assertEqual(outcomes[0], {"kind": "nan"})

    def test_scaled_residual_and_different_valid_roots(self):
        case = self.case()
        case["input"] = {"function": "irr", "values": [1e308, -1.5e308, 5e307]}
        case["expected"] = {"kind": "irr_root", "residual_atol": 1e-12}
        rust = {"kind": "finite", "value": 0}
        python = {"kind": "finite", "value": -0.5}
        row = self.row(case, rust)
        row["normalized_irr_residual"] = suite.normalized_irr_residual(case["input"], rust)
        result = suite.compare(row, {
            "outcome": python,
            "normalized_irr_residual": suite.normalized_irr_residual(case["input"], python),
        })
        self.assertEqual(result["classification"], "different_valid_irr_roots")

    def test_main_writes_failure_report_and_exits_nonzero(self):
        case = self.case()
        rust = self.row(case, {"kind": "finite", "value": 1})
        python = {"outcome": {"kind": "finite", "value": 2}, "warnings": [], "normalized_irr_residual": None}
        cargo = suite.subprocess.CompletedProcess([], 0, stdout="", stderr="")
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(suite, "run_rust", return_value=({"crate_version": "test", "cases": [rust]}, cargo)),
            patch.object(suite, "evaluate_python", return_value=python),
            patch.object(suite, "command_output", return_value="test"),
            redirect_stdout(StringIO()),
        ):
            path = Path(directory) / "report.json"
            self.assertEqual(suite.main(["--output", str(path)]), 1)
            report = json.loads(path.read_text())
            self.assertEqual(report["summary"]["failed"], 1)
            self.assertEqual(len(report["cases"]), report["summary"]["total"])
            self.assertEqual(report["cases"][0]["classification"], "unexpected_result")

    def test_cargo_failure_is_not_hidden_by_matching_results(self):
        case = self.case()
        rust = self.row(case, {"kind": "finite", "value": 1})
        python = {"outcome": rust["outcome"], "warnings": [], "normalized_irr_residual": None}
        cargo = suite.subprocess.CompletedProcess([], 101, stdout="Rust test failed", stderr="")
        output = StringIO()
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(suite, "run_rust", return_value=({"crate_version": "test", "cases": [rust]}, cargo)),
            patch.object(suite, "evaluate_python", return_value=python),
            patch.object(suite, "command_output", return_value="test"),
            redirect_stdout(output),
        ):
            path = Path(directory) / "report.json"
            self.assertEqual(suite.main(["--output", str(path)]), 1)
            report = json.loads(path.read_text())
            self.assertEqual(report["rust_execution"]["exit_code"], 101)
            self.assertEqual(report["summary"]["failed"], 0)
            self.assertIn("Rust validation failed", output.getvalue())
            self.assertIn("Rust test failed", output.getvalue())


if __name__ == "__main__":
    unittest.main()

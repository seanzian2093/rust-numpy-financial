"""Generate random cash flow series, record numpy_financial's irr, and emit a CSV
for the Rust side to check against.

    .venv/bin/python nfinancial/irr_difftest.py
"""

import random
import warnings
from pathlib import Path

import numpy as np
import numpy_financial as npf

OUT = Path(__file__).with_name("irr_cases.csv")


def make_cases(seed: int = 12345):
    rng = random.Random(seed)
    cases = []

    # Conventional: one negative outlay then positive inflows.
    for _ in range(300):
        n = rng.randint(2, 40)
        cases.append(
            (
                "conventional",
                [-rng.uniform(100, 10000)] + [rng.uniform(1, 3000) for _ in range(n)],
            )
        )

    # Non-conventional: signs assigned at random, so multiple IRRs are common.
    for _ in range(600):
        n = rng.randint(3, 25)
        v = [rng.uniform(-1000, 1000) for _ in range(n)]
        if all(x > 0 for x in v) or all(x < 0 for x in v):
            continue
        cases.append(("random-sign", v))

    # Alternating blocks, a classic multiple-IRR shape.
    for _ in range(300):
        n = rng.randint(2, 10)
        v = []
        for i in range(n):
            mag = rng.uniform(50, 2000)
            v.append(-mag if i % 2 == 0 else mag)
        cases.append(("alternating", v))

    # Near-zero and extreme magnitudes.
    for _ in range(200):
        n = rng.randint(3, 15)
        scale = 10 ** rng.randint(-4, 6)
        v = [rng.uniform(-1, 1) * scale for _ in range(n)]
        if all(x > 0 for x in v) or all(x < 0 for x in v):
            continue
        cases.append(("extreme-scale", v))

    # Built to have closely spaced root pairs: a polynomial with deliberately
    # near-equal roots, expanded back into coefficients.
    for _ in range(400):
        k = rng.randint(2, 6)
        roots = []
        for _ in range(k):
            base = rng.uniform(0.3, 3.0)
            roots.append(base)
            if rng.random() < 0.7:
                # Partner root a hair away, the case a coarse scan misses.
                roots.append(
                    base * (1 + rng.choice([1e-2, 1e-3, 1e-4]) * rng.choice([-1, 1]))
                )
        coeffs = np.poly(np.array(roots))  # highest degree first
        v = list(coeffs[::-1])  # our convention: index = period
        if all(x > 0 for x in v) or all(x < 0 for x in v):
            continue
        cases.append(("near-double-roots", [float(x) for x in v]))

    # Long conventional series.
    for _ in range(100):
        n = rng.randint(50, 400)
        cases.append(
            (
                "long-conventional",
                [-rng.uniform(1000, 50000)] + [rng.uniform(1, 2000) for _ in range(n)],
            )
        )

    return cases


def main() -> None:
    cases = make_cases()
    rows, nan_count = [], 0

    for family, v in cases:
        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")
                r = npf.irr(np.array(v))
        except Exception:
            r = float("nan")
        if r is None or np.isnan(r):
            nan_count += 1
            r = float("nan")
        rows.append(f"{family},{r!r}," + ",".join(repr(x) for x in v))

    OUT.write_text("\n".join(rows) + "\n")
    print(f"numpy_financial {npf.__version__}")
    print(f"wrote {len(rows)} cases to {OUT} ({nan_count} nan)")


if __name__ == "__main__":
    main()

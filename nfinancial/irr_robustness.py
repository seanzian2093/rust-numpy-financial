"""Probe numpy_financial.irr with level annuities of known IRR.

    pip install numpy numpy-financial
    python irr_probe.py
"""

import warnings

import numpy as np
import numpy_financial as npf

TARGET = 0.08
PRINCIPAL = 10_000.0


def annuity(length: int, rate: float = TARGET) -> np.ndarray:
    """Cash flows whose IRR is exactly `rate` by construction."""
    n = length - 1
    payment = PRINCIPAL * rate / (1.0 - (1.0 + rate) ** -n)
    return np.array([-PRINCIPAL] + [payment] * n)


def probe(length: int):
    """Return (status, value) for one series."""
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            value = npf.irr(annuity(length))
    except Exception as exc:
        return "RAISED", repr(exc)

    if value is None or np.isnan(value):
        return "NO ROOT", value
    if abs(value - TARGET) < 1e-6:
        return "OK", value
    return "WRONG", value


def main() -> None:
    print(f"numpy_financial {npf.__version__}, numpy {np.__version__}\n")

    # Boundary cases, including the lengths that broke the Rust solver.
    print("length   status    value")
    for length in (3, 4, 7, 15, 16, 17, 64, 257, 1024):
        status, value = probe(length)
        print(f"{length:>6}   {status:<8}  {value}")

    # Sweep for an aggregate count.
    counts: dict[str, int] = {}
    for length in range(3, 1025):
        status, _ = probe(length)
        counts[status] = counts.get(status, 0) + 1
    print("\nlengths 3..1024:", counts)

    # Reference case used in this crate's tests.
    reference = [-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0]
    print("\nreference series:", npf.irr(reference))
    print("expected:         0.052432888859413884")


if __name__ == "__main__":
    main()

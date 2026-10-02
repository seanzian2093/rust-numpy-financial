use rfinancial::*;

const TARGET: f64 = 0.08;
const PRINCIPAL: f64 = 10_000.0;

/// Cash flows whose IRR is exactly `rate` by construction.
fn annuity(len: usize, rate: f64) -> Vec<f64> {
    let n = (len - 1) as f64;
    let payment = PRINCIPAL * rate / (1.0 - (1.0 + rate).powf(-n));
    let mut values = vec![-PRINCIPAL];
    values.extend(std::iter::repeat(payment).take(len - 1));
    values
}

fn main() {
    // Boundary cases. Lengths 7, 15 and 17 previously returned a negative root near -1.8,
    // and length 16 panicked.
    println!("length   irr");
    for len in [3usize, 4, 7, 15, 16, 17, 64, 257, 1024] {
        match irr(&annuity(len, TARGET)) {
            Ok(Some(r)) => println!("{len:>6}   {r:.12}"),
            Ok(None) => println!("{len:>6}   no root"),
            Err(e) => println!("{len:>6}   error: {e}"),
        }
    }

    // Sweep every length, including sizes where the raw polynomial would overflow.
    let mut wrong = 0;
    for len in 3..=2048 {
        match irr(&annuity(len, TARGET)) {
            Ok(Some(r)) if (r - TARGET).abs() < 1e-6 => {}
            other => {
                wrong += 1;
                if wrong <= 5 {
                    println!("len {len}: {other:?}");
                }
            }
        }
    }
    println!("\nlengths 3..2048: {wrong} incorrect out of 2046");

    // numpy_financial 1.0.0 returns this exact value; the 0.052432888859413884 in the test
    // suite comes from an older npf release.
    let reference = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];
    println!(
        "\nreference series: {:?}",
        irr(&reference).unwrap().unwrap()
    );
    println!("numpy_financial:  0.052432888859414106");

    // Root lies at x = 1+r = 0.576, the only case below 1, so it covers the low side of the
    // bracket. A rate below -100% would be invalid: the root must satisfy x > 0.
    let loss = vec![-1000.0, 100.0, 100.0, 100.0];
    let r = irr(&loss).unwrap().unwrap();
    println!(
        "\nloss-making series: {r:.12} (npv at that rate: {:e})",
        npv(&loss, r).unwrap()
    );
    assert!(r > -1.0);

    // Validation errors are reported, not panicked.
    println!("\nall-positive: {:?}", irr(&[100.0, 200.0]));
    println!("too short:    {:?}", irr(&[-100.0]));
}

use rfinancial::*;

fn main() {
    let values: Vec<f64> = vec![-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0];

    // Function-based
    let result_f = irr(&values);
    println!("\nirr is {:?}", result_f);

    // Struct-based
    let result_s = InternalRateReturn::from_vec(values).expect("Error creating InternalRateReturn");
    println!("\n{:#?}'s irr is {:?}", result_s, result_s.get());
}

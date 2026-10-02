use rfinancial::*;

fn main() {
    let values = vec![-15000.0, 1500.0, 2500.0, 3500.0, 4500.0, 6000.0];

    // Function-based
    let result_f = npv(&values, 0.05);
    println!("\nnpv is {:?}", result_f);

    // Struct-based
    let result_s =
        NetPresentValue::from_tuple((values, 0.05)).expect("Error creating NetPresentValue");
    println!("\n{:#?}'s npv is {:?}", result_s, result_s.get());
}

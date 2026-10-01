use rfinancial::*;
fn main() {
    // Function-based
    let result_f = fv(0.075, 20, -2000.0, 0.0, WhenType::End);
    println!("fv is {:?}", result_f);

    // Struct-based
    let result_s = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End)).expect("Error creating FutureValue");
    println!("{:#?}'s fv is {:?}", result_s, result_s.get());
}
use rfinancial::*;

fn main() {
    // Function-based
    let result_f = pv(0.075, 20, -2000.0, 0.0, WhenType::End);
    println!("\npv is {:?}", result_f);

    // Struct-based
    let result_s= PresentValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End))
        .expect("Error creating PresentValue");
    println!("\n{:#?}'s pv is {:?}", result_s, result_s.get());
}

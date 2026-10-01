use rfinancial::*;

fn main() {
    // Function-based
    let result = pv(0.075, 20, -2000.0, 0.0, WhenType::End);
    println!("\npv is {:?}", result);

    // Struct-based
    let pv = PresentValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End))
        .expect("Error creating PresentValue");
    println!("\n{:#?}'s pv is {:?}", pv, pv.get());
}

use rfinancial::*;

fn main() {
    // Function-based
    let result_f = nper(0.075, -2000.0, 0.0, 100000.0, WhenType::End);
    println!("\nnper is {:?}", result_f);

    // Struct-based
    let result_s = NumberPeriod::from_tuple((0.075, -2000.0, 0.0, 100000.0, WhenType::End))
        .expect("Error creating NumberPeriod");
    println!("\n{:#?}'s nper is {:?}", result_s, result_s.get());
}

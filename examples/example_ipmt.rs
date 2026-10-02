use rfinancial::*;

fn main() {
    // Function-based
    let result_f = ipmt(0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End);
    println!("\nipmt is {:?}", result_f);

    // Struct-based
    let result_s = InterestPayment::from_tuple((0.1 / 12.0, 1, 24, 2000.0, 0.0, WhenType::End))
        .expect("Error creating InterestPayment");
    println!("\n{:#?}'s ipmt is {:?}", result_s, result_s.get());
}

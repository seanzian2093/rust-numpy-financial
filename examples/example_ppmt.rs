use rfinancial::*;

fn main() {
    // Function-based
    let result_f = ppmt(0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::End);
    println!("\nppmt is {:?}", result_f);

    // Struct-based
    let result_s = PrincipalPayment::from_tuple((0.1 / 12.0, 1, 60, 55000.0, 0.0, WhenType::End))
        .expect("Error creating PrincipalPayment");
    println!("\n{:#?}'s ppmt is {:?}", result_s, result_s.get());
}

use rfinancial::*;

fn main() {
    // Function-based
    let result_f = pmt(0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End);
    println!("\npmt is {:?}", result_f);

    // Struct-based
    let result_s = Payment::from_tuple((0.08 / 12.0, 60, 15000.0, 0.0, WhenType::End))
        .expect("Error creating Payment");
    println!("\n{:#?}'s pmt is {:?}", result_s, result_s.get());
}

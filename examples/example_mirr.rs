use rfinancial::*;

fn main() {
    let values = vec![100.0, 200.0, -50.0, 300.00, -200.0];

    // Function-based
    let result_f = mirr(&values, 0.05, 0.06);
    println!("\nmirr is {:#?}", result_f);

    // Struct-based
    let result_s =
        ModifiedIRR::from_tuple((values, 0.05, 0.06)).expect("Error creating ModifiedIRR");
    println!("\n{:#?}'s mirr is {:#?}", result_s, result_s.get());
}

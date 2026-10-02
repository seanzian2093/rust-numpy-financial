use rfinancial::*;

fn main() {
    // Function-based
    let result_f = rate(10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100);
    println!("\nrate is {:#?}", result_f);

    // Struct-based
    let result_s = Rate::from_tuple((10, 0.0, -3500.0, 10000.0, WhenType::End, 0.1, 1e-6, 100))
        .expect("Error creating Rate");
    println!("\n{:#?}'s rate is {:#?}", result_s, result_s.get());
}

fn main() {
    let mut context = fend_core::Context::new();
    println!("30% of 20: {:?}", fend_core::evaluate("30% of 20", &mut context).map(|res| res.get_main_result().to_string()));
    println!("30 percent of 20: {:?}", fend_core::evaluate("30 percent of 20", &mut context).map(|res| res.get_main_result().to_string()));
    println!("20 usd to inr: {:?}", fend_core::evaluate("20 usd to inr", &mut context).map(|res| res.get_main_result().to_string()));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", beater_api::openapi::openapi_json_pretty()?);
    Ok(())
}

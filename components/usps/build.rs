fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=usps-engine.wick");
    wick_component_codegen::configure().generate("usps-engine.wick")?;
    Ok(())
}

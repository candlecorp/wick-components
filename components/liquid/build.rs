fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=liquid.wick");
    wick_component_codegen::configure().generate("liquid.wick")?;
    Ok(())
}

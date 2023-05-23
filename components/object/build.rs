fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=object.wick");
    wick_component_codegen::configure().generate("object.wick")?;
    Ok(())
}

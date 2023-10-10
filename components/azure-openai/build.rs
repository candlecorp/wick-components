fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=component.wick");
    println!("cargo:rerun-if-changed=types.wick");
    wick_component_codegen::configure().generate("component.wick")?;
    Ok(())
}

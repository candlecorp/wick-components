fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=loop.wick");
    wick_component_codegen::configure().generate("loop.wick")?;
    Ok(())
}

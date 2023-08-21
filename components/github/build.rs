fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=github-driver.wick");
    wick_component_codegen::configure().generate("github-driver.wick")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=oauth-engine.wick");
    println!("cargo:rerun-if-changed=oauth-http.wick");
    println!("cargo:rerun-if-changed=oauth-db.wick");
    wick_component_codegen::configure().generate("oauth-engine.wick")?;
    Ok(())
}

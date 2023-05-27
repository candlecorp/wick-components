fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=oauth.wick");
  wick_component_codegen::configure().generate("oauth.wick")?;
  Ok(())
}

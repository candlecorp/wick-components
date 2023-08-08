use liquid::ParserBuilder;
use serde_json::Value;

mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[wick_component::operation(unary_simple)]
fn render(input: Value, ctx: Context<render::Config>) -> Result<String, anyhow::Error> {
    // Create a Liquid parser
    let parser = ParserBuilder::with_stdlib().build().unwrap();

    // Parse the template string
    let liquid_template = match parser.parse(&ctx.config.template) {
        Ok(t) => t,
        Err(e) => return Err(anyhow::anyhow!("Invalid template string: {}", e)),
    };

    let context = liquid::object!({"input": input});

    // Render the template with the provided parameters
    let rendered = match liquid_template.render(&context) {
        Ok(r) => r,
        Err(e) => return Err(anyhow::anyhow!("Error rendering template: {}", e)),
    };

    // Send the rendered string to the output
    Ok(rendered)
}

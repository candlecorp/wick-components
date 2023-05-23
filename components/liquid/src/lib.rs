use liquid::ParserBuilder;
use serde_json::Value;
use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl OpRender for Component {
    async fn render(
        mut input: WickStream<Value>,
        mut outputs: OpRenderOutputs,
        ctx: Context<OpRenderConfig>,
    ) -> wick::Result<()> {
        let template_string = ctx.config.template.clone();
        println!("Template: {:?}", template_string);

        // Create a Liquid parser
        let parser = ParserBuilder::with_stdlib().build().unwrap();

        while let Some(Ok(input)) = input.next().await {
            // Parse the template string
            let liquid_template = match parser.parse(&template_string) {
                Ok(t) => t,
                Err(e) => {
                    return Err(wick_component::anyhow::anyhow!(
                        "Invalid template string: {}",
                        e
                    ))
                }
            };

            println!("params: {:?}", input);

            let globals = liquid::object!(&input);

            println!("globals: {:?}", globals);

            // Render the template with the provided parameters
            let rendered = match liquid_template.render(&globals) {
                Ok(r) => r,
                Err(e) => {
                    return Err(wick_component::anyhow::anyhow!(
                        "Error rendering template: {}",
                        e
                    ))
                }
            };

            // Send the rendered string to the output
            outputs.output.send(&rendered);
        }

        // Signal that the output stream is done
        outputs.output.done();
        Ok(())
    }
}

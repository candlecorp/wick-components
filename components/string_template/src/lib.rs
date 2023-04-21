use liquid::ParserBuilder;
use serde_json::Value;
use std::collections::HashMap;
use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use wick::*;

// Implement the render operation for the StringTemplate component
#[async_trait::async_trait(?Send)]
impl OpRender for Component {
    async fn render(
        mut template: WickStream<String>,
        mut parameters: WickStream<HashMap<String, Value>>,
        mut outputs: OpRenderOutputs,
    ) -> wick::Result<()> {
        // Create a Liquid parser
        let parser = ParserBuilder::with_stdlib().build().unwrap();

        while let (Some(Ok(template_string)), Some(Ok(params))) =
            (template.next().await, parameters.next().await)
        {
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

            println!("params: {:?}", params);

            let globals = liquid::object!(&params);

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

// Here is the component definition
// - name: render
//   inputs:
//     - name: template
//       type: string
//     - name: parameters
//       type:
//         map:
//           key: string
//           value: any
//   outputs:
//     - name: output
//       type: string

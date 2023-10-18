use liquid::ParserBuilder;

mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[cfg(not(debug_assertions))]
static TEMPLATE: &str = include_str!("readme.liquid");

#[wick_component::operation(unary_simple)]
fn generate_readme(
    input: String,
    _ctx: Context<generate_readme::Config>,
) -> Result<String, anyhow::Error> {
    #[cfg(debug_assertions)]
    unimplemented!();
    #[cfg(not(debug_assertions))]
    render(input, TEMPLATE)
}

fn render(input: String, template: &str) -> anyhow::Result<String> {
    // Create a Liquid parser
    let parser = ParserBuilder::with_stdlib().build()?;

    // Parse the template string
    let liquid_template = parser.parse(template)?;

    let data: serde_json::Value = serde_yaml::from_str(&input)?;

    let context = liquid::object!({"data": data});

    // Render the template with the provided parameters
    let rendered = match liquid_template.render(&context) {
        Ok(r) => r,
        Err(e) => return Err(anyhow::anyhow!("Error rendering template: {}. If you passed a valid wick configuration, please submit it as a bug to the component's issue tracker.", e)),
    };

    // Send the rendered string to the output
    Ok(rendered)
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use wick_component::anyhow::bail;

    #[test]
    fn test_basic() -> Result<()> {
        let tpl = std::fs::read_to_string("src/readme.liquid")?;
        let components = std::fs::read_dir("..")?;
        for component in components {
            let component = component?;
            let path = component.path();
            if path.is_dir() {
                let config = path.join("component.wick");
                if config.exists() {
                    let input = std::fs::read_to_string(config)?;
                    let output = match render(input, &tpl) {
                        Ok(output) => output,
                        Err(e) => {
                            println!("Config for {} causes errors", path.display());
                            println!("error rendering template: {}", e);
                            bail!("{e}");
                        }
                    };
                    println!("{}", output);
                }
            }
        }

        Ok(())
    }
}

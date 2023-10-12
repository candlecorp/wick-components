use serde_xml_rs::from_str;
use std::collections::HashMap;
use std::collections::LinkedList;

mod wick {
    wick_component::wick_import!();
}
use jsonpath_lib::select;
use wick::*;

// Implement the "new" operation
#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl new::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = new::Inputs;
    type Outputs = new::Outputs;
    type Config = new::Config;
    async fn new(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let key = &ctx.config.key;
        while let Some(value) = inputs.value.next().await {
            let value = propagate_if_error!(value.decode(), outputs, continue);

            let mut new_object = HashMap::new();
            new_object.insert(key, value);
            let new_object_json = wick_component::to_value(new_object)?;
            outputs.output.send(&new_object_json);
        }
        outputs.output.done();
        Ok(())
    }
}

// Implement the "select" operation
#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl select::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = select::Inputs;
    type Outputs = select::Outputs;
    type Config = select::Config;
    async fn select(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let path = &ctx.config.path;
        while let Some(input) = inputs.input.next().await {
            let input = propagate_if_error!(input.decode(), outputs, continue);
            let selected_values = select(&input, &path)
                .map_err(|e| anyhow::anyhow!("Error selecting value by path: {}", e))?;

            if let Some(first_selected_value) = selected_values.first() {
                outputs.output.send(*first_selected_value);
            } else {
                outputs.output.done();
                return Err(anyhow::anyhow!("No value found at the specified path"));
            }
        }

        // Signal that the output stream is done
        outputs.output.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl serialize::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = serialize::Inputs;
    type Outputs = serialize::Outputs;
    type Config = serialize::Config;
    async fn serialize(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        while let (Some(content_string), Some(content_type_string)) = (
            inputs.content.next().await,
            inputs.content_type.next().await,
        ) {
            let content_string = propagate_if_error!(content_string.decode(), outputs, continue);
            let content_type_string =
                propagate_if_error!(content_type_string.decode(), outputs, continue);
            // Parse the content based on the content type
            let parsed_content: Value = match content_type_string.as_str() {
                "application/json" => {
                    let content_str = content_string.as_str();
                    wick_component::from_str(content_str)
                        .map_err(|e| anyhow::anyhow!("Error parsing JSON content: {}", e))?
                }
                "application/x-www-form-urlencoded" => {
                    let params = url::form_urlencoded::parse(content_string.as_bytes());
                    let parsed_params: HashMap<String, String> = params.into_owned().collect();
                    // Convert the parsed params to a serde_json::Value
                    wick_component::to_value(parsed_params)
                        .map_err(|e| anyhow::anyhow!("Error converting params to JSON: {}", e))?
                }
                "application/xml" => {
                    let content_str = content_string.as_str();
                    let parsed: HashMap<String, Value> =
                        from_str::<HashMap<String, Value>>(content_str).or_else(|_| {
                            // Wrap the content with a root element and try parsing again
                            let wrapped_content = format!("<root>{}</root>", content_str);
                            from_str(&wrapped_content)
                                .map_err(|e| anyhow::anyhow!("Error parsing XML content: {}", e))
                        })?;

                    // Flatten the nested "$value" fields
                    let mut flattened_parsed = HashMap::new();
                    for (key, value) in parsed.iter() {
                        if let Value::Object(ref inner_map) = value {
                            if let Some(inner_value) = inner_map.get("$value") {
                                flattened_parsed.insert(key.to_string(), inner_value.clone());
                            }
                        }
                    }

                    wick_component::to_value(flattened_parsed)
                        .map_err(|e| anyhow::anyhow!("Error converting XML to JSON: {}", e))?
                }
                "text/plain" => {
                    //turn content_string into serde_json::Value
                    wick_component::to_value(content_string)
                        .map_err(|e| anyhow::anyhow!("Error converting text to JSON: {}", e))?
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported content type: {}",
                        content_type_string
                    ))
                }
            };

            // Send the parsed content to the output
            outputs.output.send(&parsed_content);
        }

        // Signal that the output stream is done
        outputs.output.done();
        Ok(())
    }
}

fn extend_object_at_path(root: &mut Value, mut path: LinkedList<&str>, new_value: Value) {
    if let Some(segment) = path.pop_front() {
        if !root.as_object().unwrap().contains_key(segment) {
            root.as_object_mut()
                .unwrap()
                .insert(segment.into(), Value::Object(serde_json::Map::new()));
        }
        let next = root.get_mut(segment).unwrap();
        extend_object_at_path(next, path, new_value);
    } else {
        match new_value {
            Value::Object(new_map) => {
                if let Value::Object(map) = root {
                    for (k, v) in new_map {
                        map.insert(k, v);
                    }
                } else {
                    panic!("Root value must be an object when new value is an object");
                }
            }
            Value::String(new_string) => {
                *root = Value::String(new_string);
            }
            _ => {
                panic!("New value must be an object or a string");
            }
        }
    }
}

// Implement the "push" operation
#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl push::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = push::Inputs;
    type Outputs = push::Outputs;
    type Config = push::Config;
    async fn push(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let path = &ctx.config.path;
        while let (Some(input), Some(value)) =
            (inputs.input.next().await, inputs.value.next().await)
        {
            let mut input = propagate_if_error!(input.decode(), outputs, continue);
            let value = propagate_if_error!(value.decode(), outputs, continue);
            let path = path.replace("$.", "");
            let path: LinkedList<&str> = path.trim_start_matches('.').split('.').collect();
            extend_object_at_path(&mut input, path, value);
            outputs.output.send(&input);
        }
        outputs.output.done();
        Ok(())
    }
}

// Here is the component definition
// - name: serialize
//   inputs:
//     - name: content
//       type: string
//     - name: content_type
//       type: string
//   outputs:
//     - name: output
//       type: Value
// - name: new
//   inputs:
//     - name: key
//       type: string
//     - name: value
//       type: Value
//   outputs:
//     - name: output
//       type: Value
// - name: select
//   inputs:
//     - name: object
//       type: Value
//     - name: path
//       type: string
//   outputs:
//     - name: output
//       type: Value

use serde_json::Value;
use serde_xml_rs::from_str;
use std::collections::HashMap;
use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use jsonpath_lib::select;
use wick::*;

// Implement the "new" operation
#[async_trait::async_trait(?Send)]
impl OpNew for Component {
    async fn new(
        mut key: WickStream<String>,
        mut value: WickStream<Value>,
        mut outputs: OpNewOutputs,
    ) -> wick::Result<()> {
        if let (Some(Ok(key)), Some(Ok(value))) = (key.next().await, value.next().await) {
            let mut new_object = HashMap::new();
            new_object.insert(key, value);
            let new_object_json = serde_json::to_value(new_object)?;
            outputs.output.send(&new_object_json);
        }
        outputs.output.done();
        Ok(())
    }
}

// Implement the "select" operation
#[async_trait::async_trait(?Send)]
impl OpSelect for Component {
    async fn select(
        mut object: WickStream<Value>,
        mut path: WickStream<String>,
        mut outputs: OpSelectOutputs,
    ) -> wick::Result<()> {
        while let (Some(Ok(object_value)), Some(Ok(path_string))) =
            (object.next().await, path.next().await)
        {
            let selected_values = select(&object_value, &path_string).map_err(|e| {
                wick_component::anyhow::anyhow!("Error selecting value by path: {}", e)
            })?;

            if let Some(first_selected_value) = selected_values.first() {
                outputs.output.send(first_selected_value);
            } else {
                outputs.output.done();
                return Err(wick_component::anyhow::anyhow!(
                    "No value found at the specified path"
                ));
            }
        }

        // Signal that the output stream is done
        outputs.output.done();
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl OpSerialize for Component {
    async fn serialize(
        mut content: WickStream<String>,
        mut content_type: WickStream<String>,
        mut outputs: OpSerializeOutputs,
    ) -> wick::Result<()> {
        while let (Some(Ok(content_string)), Some(Ok(content_type_string))) =
            (content.next().await, content_type.next().await)
        {
            // Parse the content based on the content type
            let parsed_content: Value = match content_type_string.as_str() {
                "application/json" => {
                    let content_str = content_string.as_str();
                    serde_json::from_str(content_str).map_err(|e| {
                        wick_component::anyhow::anyhow!("Error parsing JSON content: {}", e)
                    })?
                }
                "application/x-www-form-urlencoded" => {
                    let params = url::form_urlencoded::parse(content_string.as_bytes());
                    let parsed_params: HashMap<String, String> = params.into_owned().collect();
                    // Convert the parsed params to a serde_json::Value
                    serde_json::to_value(parsed_params).map_err(|e| {
                        wick_component::anyhow::anyhow!("Error converting params to JSON: {}", e)
                    })?
                }
                "application/xml" => {
                    let content_str = content_string.as_str();
                    let parsed: HashMap<String, Value> =
                        from_str::<HashMap<String, Value>>(content_str).or_else(|_| {
                            // Wrap the content with a root element and try parsing again
                            let wrapped_content = format!("<root>{}</root>", content_str);
                            from_str(&wrapped_content).map_err(|e| {
                                wick_component::anyhow::anyhow!("Error parsing XML content: {}", e)
                            })
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

                    serde_json::to_value(flattened_parsed).map_err(|e| {
                        wick_component::anyhow::anyhow!("Error converting XML to JSON: {}", e)
                    })?
                }
                "text/plain" => {
                    //turn content_string into serde_json::Value
                    serde_json::to_value(content_string).map_err(|e| {
                        wick_component::anyhow::anyhow!("Error converting text to JSON: {}", e)
                    })?
                }
                _ => {
                    return Err(wick_component::anyhow::anyhow!(
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

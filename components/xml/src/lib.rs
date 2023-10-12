// lib.rs

mod wick {
    wick_component::wick_import!();
}
use anyhow::Result;
use serde_xml_rs::from_str;
use wick::*;
use wick_component::{json, Value};
use xml::reader::{EventReader, XmlEvent};
use xml::ParserConfig;

fn simplify_value(val: Value) -> Value {
    match val {
        Value::Object(map) => {
            if map.len() == 1 && map.contains_key("$value") {
                map["$value"].clone()
            } else {
                Value::Object(
                    map.into_iter()
                        .map(|(k, v)| (k, simplify_value(v)))
                        .collect(),
                )
            }
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(simplify_value).collect()),
        _ => val,
    }
}

fn group_same_elements(val: Value) -> Value {
    match val {
        Value::Object(map) => {
            let mut new_map: wick_component::Map<String, Value> = wick_component::Map::new();
            for (k, v) in map {
                let v = group_same_elements(v);
                if new_map.contains_key(&k) {
                    if let Value::Array(arr) = &mut new_map[&k] {
                        arr.push(v);
                    } else {
                        let old_v = new_map.remove(&k).unwrap();
                        new_map.insert(k, json!([old_v, v]));
                    }
                } else {
                    new_map.insert(k, v);
                }
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(group_same_elements).collect()),
        _ => val,
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl xml_to_json::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = xml_to_json::Inputs;
    type Outputs = xml_to_json::Outputs;
    type Config = xml_to_json::Config;

    async fn xml_to_json(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<()> {
        while let Some(xml_string) = inputs.xml.next().await {
            let xml_string = propagate_if_error!(xml_string.decode(), outputs, continue);
            let mut reader = EventReader::new_with_config(
                xml_string.as_bytes(),
                ParserConfig::new().trim_whitespace(true),
            );
            let mut root_element_name = None;
            loop {
                match reader.next() {
                    Ok(XmlEvent::StartElement { name, .. }) => {
                        root_element_name = Some(name.local_name);
                        break;
                    }
                    Ok(XmlEvent::EndDocument) | Err(_) => break,
                    _ => (),
                }
            }
            let parsed_data: Value = from_str(&xml_string)?;
            let simplified_data = simplify_value(parsed_data);
            let grouped_data = group_same_elements(simplified_data);
            let final_data = match root_element_name {
                Some(name) => json!({ name: grouped_data }),
                None => grouped_data,
            };
            outputs.output.send(&final_data);
        }
        outputs.output.done();
        Ok(())
    }
}

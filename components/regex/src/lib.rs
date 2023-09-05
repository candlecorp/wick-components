use std::vec;

use regex::Regex;

mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[wick_component::operation(unary_simple)]
fn match_(input: String, ctx: Context<match_::Config>) -> Result<bool, std::io::Error> {
    let pattern = ctx.config.pattern.clone();

    let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid Regex Pattern: {}", e),
            ));
        }
    };

    Ok(re.is_match(&input))
}

#[wick_component::operation(unary_simple)]
fn capture(input: String, ctx: Context<capture::Config>) -> Result<Vec<String>, std::io::Error> {
    let pattern = ctx.config.pattern.clone();

    let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid Regex Pattern: {}", e),
            ));
        }
    };

    if let Some(captures) = re.captures(&input) {
        let mut captures_vec = Vec::new();
        for capture in captures.iter() {
            if let Some(capture) = capture {
                captures_vec.push(capture.as_str().to_string());
            }
        }
        return Ok(captures_vec);
    }

    Ok(vec![])
}

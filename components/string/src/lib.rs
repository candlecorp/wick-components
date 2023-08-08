mod wick {
    wick_component::wick_import!();
}
use std::convert::Infallible;

use wick::*;

#[wick_component::operation(binary_interleaved_pairs)]
fn concatenate(
    left: String,
    right: String,
    _ctx: Context<concatenate::Config>,
) -> Result<String, Infallible> {
    Ok(format!("{}{}", left, right))
}

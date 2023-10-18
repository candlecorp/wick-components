mod coco_classes;
mod model;
mod wick {
    wick_component::wick_import!();
}

use anyhow::Result;
use wick::*;

use model::Model;

#[wick_component::operation(unary_simple)]
async fn detect(image: Bytes, ctx: Context<detect::Config>) -> Result<Value> {
    let bytes = std::fs::read(&ctx.root_config().model)?;
    let model = Model::new(bytes, "n")?;

    let results = model.run(
        image.into(),
        ctx.config.confidence.unwrap_or(0.5),
        ctx.config.iou.unwrap_or(0.5),
    )?;

    Ok(serde_json::to_value(&results)?)
}

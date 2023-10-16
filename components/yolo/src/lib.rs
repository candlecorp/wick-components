mod coco_classes;
mod model;
mod wick {
    wick_component::wick_import!();
}

use anyhow::Result;
use wick::*;

use model::Model;

#[wick_component::operation(generic_raw)]
async fn detect(
    mut inputs: detect::Inputs,
    mut outputs: detect::Outputs,
    ctx: Context<detect::Config>,
) -> Result<()> {
    let bytes = std::fs::read(&ctx.root_config().model)?;
    let model = Model::new(bytes, "n")?;
    while let Some(image) = inputs.image_data.next().await {
        let image = propagate_if_error!(image.decode(), outputs, continue);

        let results = model.run(
            image.into(),
            ctx.config.confidence.unwrap_or(0.5),
            ctx.config.iou.unwrap_or(0.5),
        )?;

        outputs.output.send(&serde_json::to_value(&results)?);
    }

    Ok(())
}

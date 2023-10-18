use crate::model::structs::{report_detect, Bbox, Multiples, YoloV8};
use candle::{DType, Device, Result, Tensor};
use candle_core as candle;
use candle_nn::{Module, VarBuilder};
use serde::{Deserialize, Serialize};

// Communication to the worker happens through bincode, the model weights and configs are fetched
// on the main thread and transfered via the following structure.
#[derive(Serialize, Deserialize)]
pub(crate) struct ModelData {
    pub weights: Vec<u8>,
    pub model_size: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RunData {
    pub image_data: Vec<u8>,
    pub conf_threshold: f32,
    pub iou_threshold: f32,
}

pub(crate) struct Model {
    model: YoloV8,
}

impl Model {
    pub(crate) fn run(
        &self,
        image_data: Vec<u8>,
        conf_threshold: f32,
        iou_threshold: f32,
    ) -> Result<Vec<Vec<Bbox>>> {
        let image_data = std::io::Cursor::new(image_data);
        let original_image = image::io::Reader::new(image_data)
            .with_guessed_format()?
            .decode()
            .map_err(candle::Error::wrap)?;
        let (width, height) = {
            let w = original_image.width() as usize;
            let h = original_image.height() as usize;
            if w < h {
                let w = w * 640 / h;
                // Sizes have to be divisible by 32.
                (w / 32 * 32, 640)
            } else {
                let h = h * 640 / w;
                (640, h / 32 * 32)
            }
        };
        let image_t = {
            let img = original_image.resize_exact(
                width as u32,
                height as u32,
                image::imageops::FilterType::CatmullRom,
            );
            let data = img.to_rgb8().into_raw();
            Tensor::from_vec(
                data,
                (img.height() as usize, img.width() as usize, 3),
                &Device::Cpu,
            )?
            .permute((2, 0, 1))?
        };
        let image_t = (image_t.unsqueeze(0)?.to_dtype(DType::F32)? * (1. / 255.))?;
        let predictions = self.model.forward(&image_t)?.squeeze(0)?;
        let bboxes = report_detect(
            &predictions,
            original_image,
            width,
            height,
            conf_threshold,
            iou_threshold,
        )?;
        Ok(bboxes)
    }

    pub(crate) fn load_(weights: Vec<u8>, model_size: &str) -> Result<Self> {
        let multiples = match model_size {
            "n" => Multiples::n(),
            "s" => Multiples::s(),
            "m" => Multiples::m(),
            "l" => Multiples::l(),
            "x" => Multiples::x(),
            _ => Err(candle::Error::Msg(
                "invalid model size: must be n, s, m, l or x".to_string(),
            ))?,
        };
        let dev = &Device::Cpu;
        let vb = VarBuilder::from_buffered_safetensors(weights, DType::F32, dev)?;
        let model = YoloV8::load(vb, multiples, 80)?;
        Ok(Self { model })
    }
}

use crate::coco_classes;
use anyhow::Result;
use inner::Model as M;
use structs::Bbox;
mod inner;
mod structs;

pub(crate) struct Model {
    inner: M,
}

impl Model {
    pub(crate) fn new(data: Vec<u8>, model_size: &str) -> Result<Model> {
        let inner = M::load_(data, model_size)?;
        Ok(Self { inner })
    }

    pub(crate) fn run(
        &self,
        image: Vec<u8>,
        conf_threshold: f32,
        iou_threshold: f32,
    ) -> Result<serde_json::Value> {
        let bboxes = self.inner.run(image, conf_threshold, iou_threshold)?;
        let mut detections: Vec<(String, Bbox)> = vec![];

        for (class_index, bboxes_for_class) in bboxes.into_iter().enumerate() {
            for b in bboxes_for_class.into_iter() {
                detections.push((coco_classes::NAMES[class_index].to_string(), b));
            }
        }
        let json = serde_json::to_value(&detections)?;
        Ok(json)
    }
}

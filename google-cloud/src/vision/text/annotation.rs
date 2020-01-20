use crate::vision::api;
use crate::vision::BoundingBox;

/// Represents a text annotation, from the text detector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAnnotation {
    pub(crate) description: String,
    pub(crate) bounding_box: BoundingBox,
}

impl TextAnnotation {
    /// Get the detected text's content.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Get the detected text's bounding box.
    pub fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }
}

impl From<api::EntityAnnotation> for TextAnnotation {
    fn from(ann: api::EntityAnnotation) -> TextAnnotation {
        TextAnnotation {
            description: ann.description,
            bounding_box: BoundingBox::from(ann.bounding_poly.unwrap()),
        }
    }
}

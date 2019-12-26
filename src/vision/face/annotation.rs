use std::convert::TryFrom;

use crate::vision::api;
use crate::vision::{BoundingBox, Likelihood};

/// Represents a text annotation, from the text detector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceAnnotation {
    pub(crate) bounding_box: BoundingBox,
    pub(crate) joy_likelihood: Likelihood,
    pub(crate) sorrow_likelihood: Likelihood,
    pub(crate) anger_likelihood: Likelihood,
    pub(crate) surprise_likelihood: Likelihood,
    pub(crate) under_exposed_likelihood: Likelihood,
    pub(crate) blurred_likelihood: Likelihood,
    pub(crate) headwear_likelihood: Likelihood,
}

impl FaceAnnotation {
    /// Get the detected face's bounding box.
    pub fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    /// Get the detected face's likelihood that it expresses joy.
    pub fn joy_likelihood(&self) -> Likelihood {
        self.joy_likelihood
    }

    /// Get the detected face's likelihood that it expresses sorrow.
    pub fn sorrow_likelihood(&self) -> Likelihood {
        self.sorrow_likelihood
    }

    /// Get the detected face's likelihood that it expresses anger.
    pub fn anger_likelihood(&self) -> Likelihood {
        self.anger_likelihood
    }

    /// Get the detected face's likelihood that it expresses surprise.
    pub fn surprise_likelihood(&self) -> Likelihood {
        self.surprise_likelihood
    }

    /// Get the detected face's likelihood that it is underexposed.
    pub fn under_exposed_likelihood(&self) -> Likelihood {
        self.under_exposed_likelihood
    }

    /// Get the detected face's likelihood that it is blurred.
    pub fn blurred_likelihood(&self) -> Likelihood {
        self.blurred_likelihood
    }

    /// Get the detected face's likelihood that it have headwear.
    pub fn headwear_likelihood(&self) -> Likelihood {
        self.headwear_likelihood
    }
}

impl TryFrom<api::FaceAnnotation> for FaceAnnotation {
    type Error = ();
    fn try_from(ann: api::FaceAnnotation) -> Result<FaceAnnotation, Self::Error> {
        Ok(FaceAnnotation {
            bounding_box: BoundingBox::from(ann.bounding_poly.unwrap()),
            joy_likelihood: Likelihood::try_from(ann.joy_likelihood)?,
            sorrow_likelihood: Likelihood::try_from(ann.sorrow_likelihood)?,
            anger_likelihood: Likelihood::try_from(ann.anger_likelihood)?,
            surprise_likelihood: Likelihood::try_from(ann.surprise_likelihood)?,
            under_exposed_likelihood: Likelihood::try_from(ann.under_exposed_likelihood)?,
            blurred_likelihood: Likelihood::try_from(ann.blurred_likelihood)?,
            headwear_likelihood: Likelihood::try_from(ann.headwear_likelihood)?,
        })
    }
}

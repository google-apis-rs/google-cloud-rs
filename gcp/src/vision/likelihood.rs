use std::convert::TryFrom;

use crate::vision::api;

/// Enum representing a likelihood.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Likelihood {
    /// Unknown likelihood.
    Unknown,
    /// It is very unlikely.
    VeryUnlikely,
    /// It is unlikely.
    Unlikely,
    /// It is possible.
    Possible,
    /// It is likely.
    Likely,
    /// It is very likely.
    VeryLikely,
}

impl From<api::Likelihood> for Likelihood {
    fn from(likelihood: api::Likelihood) -> Likelihood {
        match likelihood {
            api::Likelihood::Unknown => Likelihood::Unknown,
            api::Likelihood::VeryUnlikely => Likelihood::VeryUnlikely,
            api::Likelihood::Unlikely => Likelihood::Unlikely,
            api::Likelihood::Possible => Likelihood::Possible,
            api::Likelihood::Likely => Likelihood::Likely,
            api::Likelihood::VeryLikely => Likelihood::VeryLikely,
        }
    }
}

impl TryFrom<i32> for Likelihood {
    type Error = ();
    fn try_from(likelihood: i32) -> Result<Likelihood, Self::Error> {
        match likelihood {
            0 => Ok(Likelihood::Unknown),
            1 => Ok(Likelihood::VeryUnlikely),
            2 => Ok(Likelihood::Unlikely),
            3 => Ok(Likelihood::Possible),
            4 => Ok(Likelihood::Likely),
            5 => Ok(Likelihood::VeryLikely),
            _ => Err(()),
        }
    }
}

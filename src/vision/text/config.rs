use crate::vision::api;

/// Represents the text detection's configuration.
pub struct TextDetectionConfig {
    pub(crate) language_hints: Vec<String>,
}

impl TextDetectionConfig {
    /// Add a language hint for text detection.
    /// Language detection is automatic if none specified.
    pub fn language_hint(mut self, lang: impl Into<String>) -> TextDetectionConfig {
        self.language_hints.push(lang.into());
        self
    }
}

impl Default for TextDetectionConfig {
    fn default() -> TextDetectionConfig {
        TextDetectionConfig {
            language_hints: Vec::new(),
        }
    }
}

impl From<TextDetectionConfig> for api::ImageContext {
    fn from(config: TextDetectionConfig) -> api::ImageContext {
        api::ImageContext {
            lat_long_rect: None,
            crop_hints_params: None,
            product_search_params: None,
            web_detection_params: None,
            language_hints: config.language_hints,
        }
    }
}

/// Represents the text detection's configuration.
pub struct FaceDetectionConfig {
    pub(crate) max_results: i32,
}

impl FaceDetectionConfig {
    /// Add a language hint for text detection.
    /// Language detection is automatic if none specified.
    pub fn max_results(mut self, max_results: i32) -> FaceDetectionConfig {
        self.max_results = max_results;
        self
    }
}

impl Default for FaceDetectionConfig {
    fn default() -> FaceDetectionConfig {
        FaceDetectionConfig { max_results: 10 }
    }
}

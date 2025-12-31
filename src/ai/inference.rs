//! # ML Model Inference
//!
//! ONNX model loading and inference using `ort` crate.
//! Only available when the `ml` feature is enabled.

/// ONNX model wrapper
pub struct OnnxModel {
    // session: ort::Session,
    _placeholder: (),
}

impl OnnxModel {
    /// Load model from ONNX file
    pub fn load(_path: &str) -> Result<Self, OnnxError> {
        // TODO: Implement with ort crate
        // let session = ort::Session::builder()?
        //     .with_optimization_level(GraphOptimizationLevel::Level3)?
        //     .with_intra_threads(1)?
        //     .commit_from_file(path)?;
        Ok(Self { _placeholder: () })
    }

    /// Run inference
    pub fn run(&self, _input: &ModelInput) -> Result<ModelOutput, OnnxError> {
        // TODO: Implement inference
        Ok(ModelOutput::default())
    }
}

/// Input tensor for the model
#[derive(Clone, Debug, Default)]
pub struct ModelInput {
    pub observations: Vec<f32>,
}

impl ModelInput {
    pub fn new(observations: Vec<f32>) -> Self {
        Self { observations }
    }
}

/// Output from the model
#[derive(Clone, Debug, Default)]
pub struct ModelOutput {
    pub actions: Vec<f32>,
    pub action_index: Option<usize>,
}

impl ModelOutput {
    /// Get the action with highest probability
    pub fn best_action(&self) -> Option<usize> {
        self.action_index.or_else(|| {
            self.actions
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
        })
    }
}

/// ONNX inference error
#[derive(Debug)]
pub struct OnnxError {
    pub message: String,
}

impl std::fmt::Display for OnnxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ONNX error: {}", self.message)
    }
}

impl std::error::Error for OnnxError {}

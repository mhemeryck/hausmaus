#[derive(Debug)]
/// base error class
pub struct MausError {
    message: String,
}

impl MausError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for MausError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MausError: {}", self.message)
    }
}

impl std::error::Error for MausError {}

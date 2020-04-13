pub type Json = serde_json::Value;
pub type AsyncResult<T> = Result<T, Box<dyn std::error::Error>>;

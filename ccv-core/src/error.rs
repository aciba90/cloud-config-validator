pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid yaml: {}", .0)]
    InvalidYaml(#[from] serde_yaml::Error),

    #[error("invalid json: {}", .0)]
    InvalidJson(#[from] serde_json::Error),

    #[error("invalid JsonSchema: {}", .0)]
    InvalidSchema(String),

    #[error("request error: {}", .0)]
    RequestError(#[from] reqwest::Error),

    #[error("local JsonSchema ref not found: {}", .r#ref)]
    LocalSchemaRefNotFound { r#ref: String },

    #[error("local JsonSchema with invalid name: {}", .r#ref)]
    LocalSchemaRefInvalidName { r#ref: String },
}

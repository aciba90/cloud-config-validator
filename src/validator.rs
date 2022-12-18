use jsonschema::output::{Annotations, BasicOutput, ErrorDescription, OutputUnit};
use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SCHEMA: &str = include_str!("../schemas/versions.schema.cloud-config.resolved.1.json");

#[derive(Debug, Deserialize)]
pub struct CloudConfig {
    payload: String,
}

impl CloudConfig {
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
struct ConfigAnnotation {
    description: String,
    instance_path: String,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
struct ConfigError {
    description: String,
    instance_path: String,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Validation {
    is_valid: bool,
    annotations: Vec<ConfigAnnotation>,
    errors: Vec<ConfigError>,
}

impl From<&OutputUnit<Annotations<'_>>> for ConfigAnnotation {
    fn from(output_unit: &OutputUnit<Annotations>) -> Self {
        Self {
            description: output_unit.value().to_string(),
            instance_path: output_unit.instance_location().to_string(),
        }
    }
}

impl From<&OutputUnit<ErrorDescription>> for ConfigError {
    fn from(output_unit: &OutputUnit<ErrorDescription>) -> Self {
        Self {
            description: output_unit.error_description().to_string(),
            instance_path: output_unit.instance_location().to_string(),
        }
    }
}

impl From<BasicOutput<'_>> for Validation {
    fn from(output: BasicOutput) -> Self {
        match &output {
            BasicOutput::Valid(out_annotations) => {
                let mut annotations = Vec::with_capacity(out_annotations.len());
                for annotation in out_annotations {
                    // XXX: avoid to_mut copy
                    if let Value::Object(obj) = &annotation.value().to_mut() {
                        if let Some(Value::Bool(true)) = obj.get("deprecated") {
                            let new_annotation: ConfigAnnotation = ConfigAnnotation {
                                description: "DEPRECATED".to_string(),
                                instance_path: annotation.instance_location().to_string(),
                            };
                            annotations.push(new_annotation);
                        }
                        // TODO: extract `deprecated_msg` if present
                    }
                }
                Self {
                    is_valid: true,
                    annotations,
                    errors: vec![],
                }
            }
            BasicOutput::Invalid(out_errors) => {
                let mut errors = Vec::with_capacity(out_errors.len());
                for error in out_errors {
                    errors.push(error.into());
                }
                Self {
                    is_valid: false,
                    annotations: vec![],
                    errors,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Validator {
    json_schema: JSONSchema,
}

impl Validator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // TODO: add mechanism to fetch the schema on demand from schemastore
        Validator::from_vendored_schema()
    }

    fn from_vendored_schema() -> Self {
        let schema = serde_json::from_str(SCHEMA).expect("valid json");
        Validator::try_from(&schema).unwrap()
    }

    pub fn validate(&self, inst: &Value) -> Validation {
        self.json_schema.apply(inst).basic().into()
    }
}

impl TryFrom<&Value> for Validator {
    type Error = String;

    fn try_from(schema: &Value) -> Result<Self, Self::Error> {
        let compiled = JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft4)
            .compile(schema)
            .expect("A valid schema");
        Ok(Self {
            json_schema: compiled,
        })
    }
}

#[cfg(test)]
mod test_validate {

    use jsonschema::Draft;
    use serde_json::json;

    use super::*;
    #[test]
    fn deprecated_annotations() {
        let schema = json!(
            {
                "properties": {
                  "x": {
                    "properties": {
                      "y": {
                        "type": "integer",
                        "deprecated": true
                      }
                    },
                    "additionalProperties": false
                  }
                }
              }
        );
        let compiled_schema = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .expect("A valid schema");
        let instance = json!(
            {
                "x": {"y": 1}
            }
        );
        let out = compiled_schema.apply(&instance).basic();

        let validation: Validation = out.into();
        let expected_validation = Validation {
            is_valid: true,
            annotations: vec![ConfigAnnotation {
                description: "DEPRECATED".to_string(),
                instance_path: "/x/y".to_string(),
            }],
            errors: vec![],
        };
        dbg!(&validation);
        assert_eq!(expected_validation, validation);
    }

    #[test]
    fn config_error() {
        let schema = json!(
            {
                "properties": {
                  "x": {
                    "properties": {
                      "y": {
                        "type": "integer",
                        "deprecated": true
                      }
                    },
                    "additionalProperties": false
                  }
                }
              }
        );
        let compiled_schema = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .expect("A valid schema");
        let instance = json!(
            {
                "x": {"y": 1.5}
            }
        );
        let out = compiled_schema.apply(&instance).basic();
        dbg!(&out);

        let validation: Validation = out.into();
        let expected_validation = Validation {
            is_valid: false,
            annotations: vec![],
            errors: vec![ConfigError {
                description: "1.5 is not of type \"integer\"".to_string(),
                instance_path: "/x/y".to_string(),
            }],
        };
        dbg!(&validation);
        assert_eq!(expected_validation, validation);
    }
}

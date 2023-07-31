use std::collections::VecDeque;

use crate::error::Result;
use crate::schema::Schema;
use jsonschema::output::{Annotations, BasicOutput, ErrorDescription, OutputUnit};
use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CLOUD_CONFIG_HEADER: &str = "#cloud-config";

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
    errors: VecDeque<ConfigError>,
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
                        // Build deprecation description
                        if let Some(Value::Bool(true)) = obj.get("deprecated") {
                            let mut description = String::from("Deprecated");
                            if let Some(Value::String(cloud_init_version)) =
                                obj.get("changed_version")
                            {
                                description.push_str(" in version ");
                                description.push_str(cloud_init_version.as_str());
                            }

                            if let Some(Value::String(dsc)) = obj.get("deprecated_description") {
                                description.push_str(". ");
                                description.push_str(dsc.as_str());
                            }

                            let new_annotation: ConfigAnnotation = ConfigAnnotation {
                                description,
                                instance_path: annotation.instance_location().to_string(),
                            };
                            annotations.push(new_annotation);
                        } else if let Some(Value::Bool(true)) = obj.get("changed") {
                            let mut description = String::from("Changed");

                            if let Some(Value::String(cloud_init_version)) =
                                obj.get("changed_version")
                            {
                                description.push_str(" in version ");
                                description.push_str(cloud_init_version.as_str());
                            }

                            if let Some(Value::String(dsc)) = obj.get("changed_description") {
                                description.push_str(". ");
                                description.push_str(dsc.as_str());
                            }

                            let new_annotation: ConfigAnnotation = ConfigAnnotation {
                                description,
                                instance_path: annotation.instance_location().to_string(),
                            };
                            annotations.push(new_annotation);
                        }
                    }
                }
                Self {
                    is_valid: true,
                    annotations,
                    errors: VecDeque::new(),
                }
            }
            BasicOutput::Invalid(out_errors) => {
                let mut errors = VecDeque::with_capacity(out_errors.len());
                for error in out_errors {
                    errors.push_back(error.into());
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
    pub async fn new() -> Result<Self> {
        let schema = Schema::get().await?;
        Validator::try_from(schema.schema())
    }

    pub fn from_vendored_schema() -> Result<Self> {
        let schema = Schema::from_vendored()?;
        Validator::try_from(schema.schema())
    }

    pub fn validate(&self, inst: &Value) -> Validation {
        self.json_schema.apply(inst).basic().into()
    }

    pub fn validate_yaml(&self, payload: &str) -> Result<Validation> {
        let format_error = if !payload.starts_with(CLOUD_CONFIG_HEADER) {
            Some(ConfigError {
                description: format!(
                    "Cloud-config needs to begin with \"{}\"",
                    CLOUD_CONFIG_HEADER
                ),
                instance_path: String::new(), // XXX None
            })
        } else {
            None
        };

        let payload: Value = match serde_yaml::from_str(payload) {
            Ok(p) => p,
            Err(e) => {
                return Err(crate::error::Error::InvalidYaml(e));
            }
        };
        let mut validation = self.validate(&payload);

        if let Some(format_error) = format_error {
            validation.errors.push_front(format_error);
            validation.is_valid = false;
        }
        Ok(validation)
    }
}

impl TryFrom<&Value> for Validator {
    type Error = crate::error::Error;

    fn try_from(schema: &Value) -> Result<Self, Self::Error> {
        let compiled = JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft4)
            .compile(schema);

        match compiled {
            Err(e) => Err(Self::Error::InvalidSchema(e.to_string())),
            Ok(json_schema) => Ok(Self { json_schema }),
        }
    }
}

#[cfg(test)]
mod test_validate {

    use jsonschema::Draft;
    use serde_json::json;

    use crate::error::Error;

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
                        "deprecated_description": "my description",
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
                description: "Deprecated. my description".to_string(),
                instance_path: "/x/y".to_string(),
            }],
            errors: VecDeque::new(),
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
            errors: VecDeque::from(vec![ConfigError {
                description: "1.5 is not of type \"integer\"".to_string(),
                instance_path: "/x/y".to_string(),
            }]),
        };
        dbg!(&validation);
        assert_eq!(expected_validation, validation);
    }

    #[test]
    fn invalid_yaml() {
        let validator = Validator::from_vendored_schema().unwrap();
        let validation = validator.validate_yaml("@asdf");
        let error_msg = match validation {
            Err(Error::InvalidYaml(e)) => e.to_string(),
            _ => panic!("unexpected result"),
        };
        assert_eq!(
            "found character that cannot start any token, while scanning for the next token",
            error_msg
        );
    }
}

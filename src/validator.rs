use jsonschema::output::BasicOutput;
use jsonschema::JSONSchema;
use serde_json::value::Value;

const SCHEMA: &str = include_str!("../schemas/versions.schema.cloud-config.json");

#[derive(Debug)]
pub struct Validator {
    json_schema: JSONSchema,
}

impl Validator {
    pub fn new() -> Self {
        let schema = serde_json::from_str(SCHEMA).expect("valid json");
        Validator::try_from(&schema).unwrap()
    }

    pub fn validate(&self, inst: &Value) -> BasicOutput {
        self.json_schema.apply(inst).basic()
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

    fn validator() -> Validator {
        let schema = serde_json::from_str(SCHEMA).expect("valid json");
        Validator::try_from(&schema).unwrap()
    }

    fn basic_test(instance: &Value) {
        let validator = validator();
        // dbg!(&validator);
        match validator.validate(instance) {
            BasicOutput::Valid(annotations) => {
                for annotation in annotations {
                    println!(
                        "Value: {} at path {}",
                        annotation.value(),
                        annotation.instance_location()
                    )
                }
            }
            BasicOutput::Invalid(errors) => {
                dbg!(&errors);
                for error in errors {
                    println!(
                        "Error: {} at path {}",
                        error.error_description(),
                        error.instance_location()
                    )
                }
            }
        }
        panic!("asf");
    }

    #[test]
    fn test_1() {
        let instance = json!(
            {"ubuntu_advantage": {"token": "win", "invalidkey": ""}});
        basic_test(&instance);
    }

    #[test]
    fn test_2() {
        let instance = json!(
            {
                    "ubuntu_advantage": {
                        "enable": ["fips"],
                        "token": "<token>",
                        "config": ["http_proxy=http://some-proxy:8088"],
                    }
                }
        );
        basic_test(&instance);
    }
    /*
    *
            # Strict keys
            pytest.param(
                {"ubuntu_advantage": {"token": "win", "invalidkey": ""}},
                pytest.raises(
                    SchemaValidationError,
                    match=re.escape(
                        "ubuntu_advantage: Additional properties are not"
                        " allowed ('invalidkey"
                    ),
                ),
                id="additional_properties",
            )
        unimplemented!();
        */
    #[test]
    fn deprecated_annotations() {
        let schema = json!(
            {
                "properties": {
                  "Name": {
                    "maxLength": 5,
                    "deprecated": true
                  },
                  "x": {
                    "properties": {
                      "y": {
                        "type": "integer"
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
                "Name": "asdfasdfasdfas",
                "x": {"y": 1.5, "z": 5}
            }
        );
        let out = compiled_schema.apply(&instance).basic();
        dbg!(&out);
        match out {
            BasicOutput::Valid(annotations) => {
                for annotation in annotations {
                    println!(
                        "Value: {} at path {}",
                        annotation.value(),
                        annotation.instance_location()
                    )
                }
            }
            BasicOutput::Invalid(errors) => {
                // dbg!(&errors);
                for error in errors {
                    println!(
                        "Error: {} at path {}",
                        error.error_description(),
                        error.instance_location()
                    )
                }
            }
        }
        panic!();
    }
}

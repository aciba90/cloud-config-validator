mod validator;

use jsonschema::output::BasicOutput;
use validator::Validator;

fn validate_test(validator: &Validator, instance: &str) {
    let instance = serde_json::from_str(instance).unwrap();
    match validator.validate(&instance) {
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
}

fn main() {
    let validator = Validator::new();
    let instance = "";
    validate_test(&validator, instance);
}

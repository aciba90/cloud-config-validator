use std::{fs, path::Path, process};

use ccv_core::validator::Validator;
use clap::Parser;

#[derive(Parser)]
#[command(name = "ccv")]
enum CCVCli {
    Validate(ValidateArgs),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct ValidateArgs {
    file: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> process::ExitCode {
    let CCVCli::Validate(args) = CCVCli::parse();

    let payload = if args.file.is_none() || Some(Path::new("-")) == args.file.as_deref() {
        todo!("read stdin");
    } else {
        let f = args.file.expect("args.file is not None");
        match fs::read_to_string(&f) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading {:?}: {}", f, e);
                return process::ExitCode::FAILURE;
            }
        }
    };
    let validator = match Validator::new().await {
        Err(e) => panic!("Error reading the JsonSchema: {}", e),
        Ok(v) => v,
    };

    // TODO handle error
    let validation = validator.validate_yaml(&(payload)).unwrap();
    let exit_code = if validation.is_valid {
        process::ExitCode::SUCCESS
    } else {
        // XXX: Unique exit code? 2 os used by clap when bad used
        process::ExitCode::FAILURE
    };

    let res = serde_json::to_value(&validation).expect("Validation must be JSON serializable");
    println!("{}", res);

    exit_code
}

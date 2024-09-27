use ccv_core::{schema::ConfigKind, validator::Validator};
use clap::builder::TypedValueParser as _;
use clap::Parser;
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process,
};

#[derive(Parser)]
#[command(name = "ccv", author, version, about, long_about = None)]
enum CCVCli {
    Validate(ValidateArgs),
}

#[derive(clap::Args)]
struct ValidateArgs {
    #[arg(default_value=PathBuf::from("-").into_os_string())]
    file: std::path::PathBuf,

    #[arg(
        long,
        default_value_t = ConfigKind::CloudConfig,
        value_parser = clap::builder::PossibleValuesParser::new(["cloudconfig", "networkconfig"])
            .map(|s| s.parse::<ConfigKind>().unwrap()),
    )]
    kind: ConfigKind,
}

#[tokio::main]
async fn main() -> process::ExitCode {
    let CCVCli::Validate(args) = CCVCli::parse();

    let payload = if Path::new("-") == args.file {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    } else {
        let f = args.file;
        match fs::read_to_string(&f) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error reading {:?}: {}", f, e);
                return process::ExitCode::FAILURE;
            }
        }
    };
    let validator = match Validator::new(args.kind).await {
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

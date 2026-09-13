use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const HELP: &str = "Ketox — safe, ergonomic Rust bindings for Kotlin

Usage:
  ketox generate --source <file.rs> --package <dev.example> --class <RustApi> --library <native_name> --out <directory>
  ketox inspect  --source <file.rs> --package <dev.example> --class <RustApi> --library <native_name>
  ketox --help
  ketox --version

Commands:
  generate  Write Kotlin, Rust JNI glue, and JSON metadata to an output directory.
  inspect   Validate exported functions and print their JSON metadata.

Options:
  --source   Rust source containing top-level #[kotlin_export] functions.
  --package  Kotlin package, for example dev.ketox.example.
  --class    Generated Kotlin object name, for example RustApi.
  --library  Native library base name without lib prefix or platform extension.
  --out      Output directory (generate only).

Phase 1 supports bool, i8, i16, i32, i64, f32, f64, String, &str inputs, and () returns.
";

struct Configuration {
    source: PathBuf,
    package: String,
    class_name: String,
    library_name: String,
}

enum Command {
    Generate(Configuration, PathBuf),
    Inspect(Configuration),
    Help,
    Version,
}

fn main() -> ExitCode {
    match parse_arguments(env::args_os().skip(1)).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ketox: {error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_arguments(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut arguments = arguments.into_iter();
    let Some(command) = arguments.next() else {
        return Ok(Command::Help);
    };
    let command = command
        .into_string()
        .map_err(|_| "command must contain valid Unicode".to_owned())?;
    if matches!(command.as_str(), "--help" | "-h" | "--version" | "-V") {
        if arguments.next().is_some() {
            return Err(format!("unexpected arguments after {command}"));
        }
        return Ok(if matches!(command.as_str(), "--help" | "-h") {
            Command::Help
        } else {
            Command::Version
        });
    }
    if !matches!(command.as_str(), "generate" | "inspect") {
        return Err(format!("unknown command '{command}'; run ketox --help"));
    }
    let mut options = BTreeMap::new();
    while let Some(flag) = arguments.next() {
        let flag = flag
            .into_string()
            .map_err(|_| "option names must contain valid Unicode".to_owned())?;
        if matches!(flag.as_str(), "--help" | "-h") {
            if !options.is_empty() || arguments.next().is_some() {
                return Err("use help by itself: ketox --help".to_owned());
            }
            return Ok(Command::Help);
        }
        let recognized = matches!(
            flag.as_str(),
            "--source" | "--package" | "--class" | "--library"
        ) || (command == "generate" && flag == "--out");
        if !recognized {
            return Err(format!(
                "unknown option '{flag}' for {command}; run ketox --help"
            ));
        }
        if options.contains_key(&flag) {
            return Err(format!("option {flag} was specified more than once"));
        }
        let value = arguments
            .next()
            .filter(|value| !value.is_empty() && !value.to_string_lossy().starts_with("--"))
            .ok_or_else(|| format!("missing value for {flag}"))?;
        options.insert(flag, value);
    }

    let mut take = |flag: &str| {
        options
            .remove(flag)
            .ok_or_else(|| format!("missing required option {flag}; run ketox --help"))
    };
    let source = PathBuf::from(take("--source")?);
    let package = text_value(take("--package")?, "--package")?;
    let class_name = text_value(take("--class")?, "--class")?;
    let library_name = text_value(take("--library")?, "--library")?;
    let configuration = Configuration {
        source,
        package,
        class_name,
        library_name,
    };
    if command == "generate" {
        Ok(Command::Generate(
            configuration,
            PathBuf::from(take("--out")?),
        ))
    } else {
        Ok(Command::Inspect(configuration))
    }
}

fn text_value(value: OsString, flag: &str) -> Result<String, String> {
    value
        .into_string()
        .map_err(|_| format!("{flag} must contain valid Unicode"))
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Help => write_stdout(HELP),
        Command::Version => write_stdout(&format!("ketox {}\n", env!("CARGO_PKG_VERSION"))),
        Command::Generate(configuration, output) => {
            ketox_codegen::generate_from_file(
                &configuration.source,
                &configuration.package,
                &configuration.class_name,
                &configuration.library_name,
                &output,
            )?;
            write_stdout(&format!(
                "Generated {}.kt, ketox_jni.rs, and ketox-metadata.json in {}\n",
                configuration.class_name,
                output.display()
            ))
        }
        Command::Inspect(configuration) => {
            let source = fs::read_to_string(&configuration.source).map_err(|error| {
                format!(
                    "could not read Rust source '{}': {error}",
                    configuration.source.display()
                )
            })?;
            let module = ketox_core::parse_source(
                &source,
                &configuration.package,
                &configuration.class_name,
                &configuration.library_name,
            )
            .map_err(|error| format!("{}: {error}", configuration.source.display()))?;
            let metadata = serde_json::to_string_pretty(&module)
                .map_err(|error| format!("could not serialize Ketox metadata: {error}"))?;
            write_stdout(&(metadata + "\n"))
        }
    }
}

fn write_stdout(value: &str) -> Result<(), String> {
    io::stdout()
        .lock()
        .write_all(value.as_bytes())
        .map_err(|error| format!("could not write standard output: {error}"))
}

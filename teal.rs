//! Teal paint main application
use std::env;
use std::process::ExitCode;
use std::error::Error;
use clap::Parser;

/// Main input arguments.
#[derive(Parser, Debug)]
#[command(version, about)]
struct TealArgs {
    /// Image file path.
    #[arg(short, long)]
    file_path: String,

    /// Optional new image dimensions (in format WIDTHxHEIGHT).
    #[arg(short, long, value_parser = parse_dims)]
    dims: Option<(u32, u32)>,
}

fn parse_dims(s: &str) -> Result<(u32, u32), Box<dyn Error + Send + Sync + 'static>> {
    let idx = s
        .find('x')
        .ok_or_else(|| format!("missing 'x' in dimension argument '{s}'"))?;
    Ok((s[..idx].parse()?, s[idx + 1..].parse()?))
}

fn main() -> ExitCode {
    let teal_args = TealArgs::parse();
    let args = teal_main::Args {
        fname: teal_args.file_path,
        dims: teal_args.dims,
    };
    let config_data = std::fs::read_to_string("./teal.toml")
        .expect("failed to read teal config");
    let config: teal_main::Config =
        toml::from_str(&config_data).expect("failed to parse teal config");
    teal_main::run(args, config, teal_gui::GtkGUI::new())
}

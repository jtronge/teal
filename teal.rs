//! Teal paint main application
use std::env;
use std::process::ExitCode;

fn usage(bin_name: &str) -> ExitCode {
    eprintln!("Usage: {bin_name} <FILE> [WIDTH] [HEIGHT]");
    ExitCode::FAILURE
}

/// Parse command line arguments.
fn parse_args(mut args: env::Args) -> Result<teal_main::Args, ()> {
    // Ensure we have a filename.
    let fname = args.next();
    if fname.is_none() {
        eprintln!("ERROR: Missing file name.");
        return Err(());
    }
    let fname = fname.unwrap();

    // Check for dimension args.
    if let Some(width) = args.next() {
        if let Some(height) = args.next() {
            if let Some(_) = args.next() {
                eprintln!("ERROR: Too many arguments.");
                return Err(());
            }

            let width = width.parse::<u32>();
            let height = height.parse::<u32>();
            if width.is_err() || height.is_err() {
                eprintln!("ERROR: Width and height dimensions are not valid.");
                return Err(());
            }

            Ok(teal_main::Args {
                fname,
                dims: Some((width.unwrap(), height.unwrap())),
            })
        } else {
            eprintln!("ERROR: Missing height dimension.");
            Err(())
        }
    } else {
        Ok(teal_main::Args { fname, dims: None })
    }
}

fn main() -> ExitCode {
    let config_data = std::fs::read_to_string("./teal.toml").expect("failed to read teal config");
    let mut args = env::args();
    let bin_name = args.next().expect("failed to get binary name");
    if let Ok(args) = parse_args(args) {
        let config: teal_main::Config =
            toml::from_str(&config_data).expect("failed to parse teal config");
        teal_main::run(args, config, teal_gui::GtkGUI::new())
    } else {
        eprintln!();
        usage(&bin_name)
    }
}

//! Teal paint main application

fn main() {
    let config_data = std::fs::read_to_string("./teal.toml")
        .expect("failed to read teal config");
    let args = teal_main::Args {
        fname: "out.exr".to_string(),
    };
    let config: teal_main::Config = toml::from_str(&config_data)
        .expect("failed to parse teal config");
    teal_main::run(args, config, teal_gui::GtkGUI::new());
}

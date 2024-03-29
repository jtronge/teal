//! Teal paint main application

fn main() {
    let config_data = std::fs::read_to_string("./teal.toml")
        .expect("failed to read teal config");
    let config: teal_main::Config = toml::from_str(&config_data)
        .expect("failed to parse teal config");
    teal_main::run(teal_gui::GtkGUI::new(), config);
}

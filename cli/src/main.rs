use std::{
    fs::OpenOptions,
    io::Write,
    process::{Command, Stdio, exit},
};

use common::achievements;
use strum::VariantArray;

static USAGE: &str = r#"
Usage:

cargo run --bin common generate-icons
"#;

fn generate_icons() -> std::io::Result<()> {
    for item in achievements::AchievementType::VARIANTS {
        println!("Processing {item:?}");

        let icon = item.get_icon_source();
        let path = format!("static/achievement-icons/{item:?}.svg");
        println!("{path}");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(icon.as_bytes())?;
        drop(file);

        Command::new("npx")
            .args(["svgo", path.as_str()])
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .output()?;
    }

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let arg = std::env::args().nth(1);

    match arg.as_deref() {
        None | Some("--help") => {
            eprintln!("{USAGE}");
            exit(1);
        }
        Some("generate-icons") => {
            generate_icons()?;
        }
        _ => {
            eprintln!("{USAGE}");
            exit(2);
        }
    }

    Ok(())
}

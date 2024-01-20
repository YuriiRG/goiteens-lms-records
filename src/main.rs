use std::{
    fs::{self, File},
    io::Write,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::json;

#[derive(Parser)]
#[command(name = "lms-records")]
#[command(author = "YuriiRG")]
#[command(version)]
#[command(about = "Uploads lesson records to GoITeens LMS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log in GoITeens admin panel, creating refresh-token.txt file
    Login {
        /// GoITeens LMS admin panel username (email)
        username: String,

        /// GoITeens LMS admin panel password
        password: String,
    },
    /// Request test data
    Test,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenResponse {
    success: bool,
    error: String,
    refresh_token: String,
    access_token: String,
}

fn main() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Login { username, password } => {
            println!("Logging in... It's going to take a long time");
            let res: TokenResponse =
                ureq::post("https://api.admin.edu.goiteens.com/api/v1/auth/login")
                    .send_json(json!({
                        "username": username,
                        "password": password,
                        "url": "https://admin.edu.goiteens.com/account/login"
                    }))
                    .context("Network error while logging in")?
                    .into_json()?;
            if !res.success {
                bail!("LMS Error: {}", res.error);
            }
            let mut file = File::create("refresh-token.txt")?;
            file.write_all(res.refresh_token.as_bytes())?;
            println!("Logged in successfully! A file named refresh-token.txt should appear.");
            println!("This file is necessary for all other commands to work");
        }
        Commands::Test => {
            let refresh_token = get_refresh_token()?;
            let access_token = get_access_token(&refresh_token)?;

            let res: serde_json::Value = ureq::get("https://api.admin.edu.goiteens.com/api/v1/training-module/additional-material/list?moduleId=17098215&groupId=17209734")
                .set("Authorization", &format!("Bearer {access_token}"))
                .call()?
                .into_json()?;

            println!("{res:#?}");
        }
    };
    Ok(())
}

fn get_access_token(refresh_token: &str) -> Result<String> {
    let res: TokenResponse = ureq::post("https://api.admin.edu.goiteens.com/api/v1/auth/refresh")
        .set("Cookie", &format!("refreshToken={refresh_token}"))
        .call()?
        .into_json()?;

    if !res.success {
        bail!("LMS Error: {}", res.error);
    }

    let mut file = File::create("refresh-token.txt")?;
    file.write_all(res.refresh_token.as_bytes())?;

    Ok(res.access_token)
}

fn get_refresh_token() -> Result<String> {
    fs::read_to_string("./refresh-token.txt").context("Could not find refresh-token.txt file")
}

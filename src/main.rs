use std::{
    env,
    fs::{self, File},
    io::Write,
};

use ahash::AHashMap;
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
    /// Quiet mode. Don't print successful actions
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log in to GoITeens admin panel, creating refresh-token.txt file
    Login {
        /// GoITeens LMS admin panel username (email)
        username: String,

        /// GoITeens LMS admin panel password
        password: String,
    },

    /// Log in to GoITeens admin panel using environment variables LMS_USERNAME and LMS_PASSWORD (.env supported)
    LoginEnv,

    /// Upload records into the LMS for a group from input.txt file
    ///
    /// input.txt has tech skills and soft skills lessons separated by double newline.
    /// Each lesson is is tab-separated line with the lesson's name and a link to its record.
    Upload { group_id: u64 },

    /// Remove all lesson records for a group
    Remove { group_id: u64 },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenResponse {
    success: bool,
    error: String,
    refresh_token: String,
    access_token: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GenericResponse {
    success: bool,
    error: String,
}

#[derive(Debug)]
struct Lesson {
    name: String,
    link: String,
}

#[derive(Deserialize)]
struct LessonListResponse {
    success: bool,
    error: String,
    group: Option<Vec<LessonResponse>>,
}

#[derive(Deserialize)]
struct LessonResponse {
    id: u64,
    name: String,
}

impl Lesson {
    fn new(name: &str, link: &str, i: Option<usize>, lesson_type: &str) -> Lesson {
        let marker = match i {
            None => "".to_string(),
            Some(i) => format!(" ({})", i + 1),
        };
        Lesson {
            name: if name.to_lowercase().contains(&lesson_type.to_lowercase())
                || name
                    .to_lowercase()
                    .contains(&lesson_type.to_lowercase().replace(' ', "_"))
            {
                format!("{}{marker}", truncate_chars(name, 70 - marker.len()))
            } else {
                format!(
                    "{}{marker}",
                    truncate_chars(&format!("{lesson_type} {name}"), 70 - marker.len())
                )
            },
            link: link.to_string(),
        }
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

fn main() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    let agent = ureq::AgentBuilder::new().build();

    match cli.command {
        Commands::Login { username, password } => {
            log_in(&username, &password, cli.quiet)?;
        }
        Commands::LoginEnv => {
            let username =
                env::var("LMS_USERNAME").context("No LMS_USERNAME environment variable found")?;
            let password =
                env::var("LMS_PASSWORD").context("No LMS_PASSWORD environment variable found")?;
            log_in(&username, &password, cli.quiet)?;
        }
        Commands::Upload { group_id } => {
            let refresh_token = get_refresh_token()?;
            let access_token = get_access_token(&refresh_token)?;

            let lessons = fs::read_to_string("./input.txt")
                .context("input.txt file not found")?
                .replace("\r\n", "\n");

            let (tech_skills, soft_skills) = lessons.split_once("\n\n").unwrap_or((&lessons, ""));

            let tech_skills =
                tech_skills
                    .lines()
                    .filter_map(|lesson| match lesson.split_once('\t') {
                        None => None,
                        Some((_, "")) => None,
                        full => full,
                    });

            let soft_skills =
                soft_skills
                    .lines()
                    .filter_map(|lesson| match lesson.split_once('\t') {
                        None => None,
                        Some((_, "")) => None,
                        full => full,
                    });

            let mut lessons = vec![];

            for (name, link) in tech_skills {
                if link.contains(' ') {
                    let links: Vec<_> = link.split(' ').collect();
                    for (i, link) in links.into_iter().enumerate() {
                        lessons.push(Lesson::new(name, link, Some(i), "Tech skills"));
                    }
                } else {
                    lessons.push(Lesson::new(name, link, None, "Tech skills"));
                }
            }

            for (name, link) in soft_skills {
                if link.contains(' ') {
                    let links: Vec<_> = link.split(' ').collect();
                    for (i, link) in links.into_iter().enumerate() {
                        lessons.push(Lesson::new(name, link, Some(i), "Soft skills"));
                    }
                } else {
                    lessons.push(Lesson::new(name, link, None, "Soft skills"));
                }
            }

            let mut lesson_counts = AHashMap::default();

            for lesson in &mut lessons {
                let count = *lesson_counts
                    .entry(lesson.name.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1u8);
                if count > 1 {
                    let marker = format!(" ({count})");
                    lesson.name = format!(
                        "{}{marker}",
                        truncate_chars(&lesson.name, 70 - marker.len())
                    );
                }
            }

            for lesson in lessons {
                let lesson_type = if lesson.link.contains("youtu") {
                    "video"
                } else {
                    "other"
                };
                let res: GenericResponse = agent.post("https://api.admin.edu.goiteens.com/api/v1/training-module/additional-material/create")
                .set("Authorization", &format!("Bearer {access_token}"))
                .send_json(json!({
                    "category": "group",
                    "type": lesson_type,
                    "moduleId": 17063573,
                    "groupId": group_id,
                    "name": lesson.name,
                    "link": lesson.link
                }))?
                .into_json()?;

                if res.success {
                    if !cli.quiet {
                        println!("Successfully uploaded lesson \"{}\"", lesson.name);
                    }
                } else {
                    bail!(
                        "When uploading lesson \"{}\" GoITeens LMS returned an error: {}",
                        lesson.name,
                        res.error
                    );
                }
            }
        }
        Commands::Remove { group_id } => {
            let refresh_token = get_refresh_token()?;
            let access_token = get_access_token(&refresh_token)?;

            let res: LessonListResponse = agent.get(&format!("https://api.admin.edu.goiteens.com/api/v1/training-module/additional-material/list?moduleId=17063573&groupId={group_id}"))
                .set("Authorization", &format!("Bearer {access_token}"))
                .call()?
                .into_json()?;

            if !res.success {
                bail!("GoITeens LMS returned an error: {}", res.error);
            }

            let lessons = res
                .group
                .context("GoITeens LMS returned an invalid response")?;

            for lesson in lessons {
                let res: GenericResponse = agent.post("https://api.admin.edu.goiteens.com/api/v1/training-module/additional-material/delete")
                    .set("Authorization", &format!("Bearer {access_token}"))
                    .send_json(json!({
                        "materialId": lesson.id
                    }))?
                    .into_json()?;
                if res.success {
                    if !cli.quiet {
                        println!("Successfully removed lesson {}", lesson.name);
                    }
                } else {
                    bail!(
                        "When removing lesson \"{}\" GoITeens LMS returned an error: {}",
                        lesson.name,
                        res.error
                    );
                }
            }
        }
    };
    Ok(())
}

fn log_in(username: &str, password: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Logging in... It's going to take a long time");
    }

    let res: TokenResponse = ureq::post("https://api.admin.edu.goiteens.com/api/v1/auth/login")
        .send_json(json!({
            "username": username,
            "password": password,
            "url": "https://admin.edu.goiteens.com/account/login"
        }))
        .context("Network error while logging in")?
        .into_json()?;

    if !res.success {
        bail!("GoITeens LMS returned an error: {}", res.error);
    }

    let mut file = File::create("refresh-token.txt")?;
    file.write_all(res.refresh_token.as_bytes())?;

    if !quiet {
        println!("Successfully logged in! A file named refresh-token.txt should appear.");
        println!("This file is necessary for all other commands to work");
    }
    Ok(())
}

fn get_access_token(refresh_token: &str) -> Result<String> {
    let res: TokenResponse = ureq::post("https://api.admin.edu.goiteens.com/api/v1/auth/refresh")
        .set("Cookie", &format!("refreshToken={refresh_token}"))
        .call()?
        .into_json()?;

    if !res.success {
        bail!("GoITeens LMS returned an error: {}", res.error);
    }

    let mut file = File::create("refresh-token.txt")?;
    file.write_all(res.refresh_token.as_bytes())?;

    Ok(res.access_token)
}

fn get_refresh_token() -> Result<String> {
    fs::read_to_string("./refresh-token.txt").context("Could not find refresh-token.txt file")
}

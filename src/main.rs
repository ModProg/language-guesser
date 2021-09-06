use std::io;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use anyhow::{anyhow, Result};
use events::Events;
use rand::prelude::*;
use serde::Deserialize;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Wrap};

use self::events::Event;
mod events;

const LANGUAGES: &[&str] = &[
    "rust",
    "javascript",
    "typescript",
    "go",
    "java",
    "kotlin",
    "dart",
    "html",
    "ruby",
    "php",
    "css",
    "c#",
    "c++",
    "c",
];

#[derive(Deserialize, Debug)]
struct CodeRequest {
    download_url: String,
}

#[derive(Debug)]
struct Code {
    repository: octocrab::models::Repository,
    code: String,
    language: usize,
}

async fn get_code(languages: &Vec<String>) -> Result<Code> {
    let rng = &mut rand::thread_rng();
    let octocrab = octocrab::instance();
    let idx = rng.gen_range(0..languages.len());
    let language = &languages[idx];

    let repos = octocrab
        .search()
        .repositories(&format!("language:{} license:mit stars:>=30", language))
        .sort("updated")
        .send()
        .await?
        .items;

    let repo = repos.choose(rng).ok_or_else(|| anyhow!(""))?;

    let files = octocrab
        .search()
        .code(&format!("language:{} repo:{}", language, repo.full_name))
        .send()
        .await?
        .items;

    let file = files.choose(rng).ok_or_else(|| anyhow!(""))?;

    let code: CodeRequest = octocrab.get(&file.url, None::<&()>).await?;
    let code: String = octocrab
        ._get(code.download_url, None::<&()>)
        .await?
        .text()
        .await?;
    Ok(Code {
        repository: repo.clone(),
        code,
        language: idx,
    })
}

const MAX_POINTS: i32 = 12;
const STEP_DURATION: Duration = Duration::from_secs(2);
fn shown_chars(points: i32) -> i32 {
    2i32.pow((MAX_POINTS - points).max(0) as u32)
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(token) = std::env::var("LANGUAGE_GUESSER_TOKEN") {
        // Set account and pw
        octocrab::initialise(octocrab::Octocrab::builder().personal_token(token))?;
    }

    // Create Terminal
    let points = {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let events = Events::new();

        let rng = &mut rand::thread_rng();

        let mut points_total = 0;
        let mut lives = 5;
        // println!("{}", get_code().await?.code);
        'main: loop {
            if lives == 0 {
                break;
            }
            let languages: Vec<String> = LANGUAGES
                .choose_multiple(rng, 4)
                .map(|f| f.to_string())
                .collect();
            let code = if let Ok(code) = get_code(&languages).await {
                code
            } else {
                continue;
            };
            let language_descriptions = languages.into_iter().zip(1..=4).collect::<Vec<_>>();
            let mut points_round = MAX_POINTS;
            let origin = rng.gen_range(0..code.code.len()) as i32;
            let mut last = Instant::now();
            loop {
                if Instant::now().duration_since(last) > STEP_DURATION {
                    if points_round == 0 {
                        lives -= 1;
                        if lives == 0 {
                            break 'main;
                        } else {
                            break;
                        }
                    } else {
                        points_round -= 1;
                    }
                    last = Instant::now();
                }
                terminal.draw(|f| {
                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(10), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(f.size());
                    let block = Block::default().borders(Borders::ALL);
                    {
                        let inner = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(
                                [
                                    Constraint::Length(2),
                                    Constraint::Length(1),
                                    Constraint::Percentage(100),
                                ]
                                .as_ref(),
                            )
                            .split(block.inner(vertical[0]));
                        let paragraph = Paragraph::new(
                            "Press CTRL+C if you want to give up.\nPress 1-4 to guess a language.",
                        );
                        f.render_widget(paragraph, inner[0]);
                        let bottom = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [
                                    Constraint::Ratio(1, 5),
                                    Constraint::Ratio(1, 5),
                                    Constraint::Ratio(1, 5),
                                    Constraint::Ratio(1, 5),
                                    Constraint::Ratio(1, 5),
                                ]
                                .as_ref(),
                            )
                            .split(inner[2]);
                        let p = Paragraph::new(format!("Total Points: {}", points_total));
                        f.render_widget(p, bottom[0]);

                        let p = Paragraph::new(format!("Round Points: {}", points_round));
                        f.render_widget(p, bottom[1]);

                        let p = Paragraph::new(format!("Lives: {}", "❦ ".repeat(lives)));
                        f.render_widget(p, bottom[2]);
                    }
                    f.render_widget(block, vertical[0]);

                    let horizontal = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(0), Constraint::Length(20)])
                        .split(vertical[1]);
                    let shown_chars = if points_round == 0 {
                        code.code.len() as i32
                    } else {
                        shown_chars(points_round)
                    };
                    let start = (code.code.len() as i32 - shown_chars)
                        .min(origin as i32 - shown_chars / 2)
                        .max(0);
                    let code = Paragraph::new(
                        code.code
                            .as_str()
                            .chars()
                            .skip(start as usize)
                            .take(shown_chars as usize)
                            .collect::<String>(),
                    )
                    .wrap(Wrap { trim: false })
                    .block(Block::default().title("Code").borders(Borders::ALL));

                    f.render_widget(code, horizontal[0]);
                    let table =
                        Table::new(language_descriptions.iter().map(|(language, number)| {
                            Row::new(vec![number.to_string(), language.to_string()])
                        }))
                        .widths(&[Constraint::Length(3), Constraint::Percentage(100)])
                        .block(Block::default().title("Languages").borders(Borders::ALL));
                    f.render_widget(table, horizontal[1]);
                })?;

                if let Event::Input(input) = events.next()? {
                    if let Key::Ctrl('c') = input {
                        break 'main;
                    }
                    if let Key::Char('1') = input {
                        if code.language == 0 {
                            points_total += points_round;
                        } else {
                            lives -= 1;
                        }
                        break;
                    }
                    if let Key::Char('2') = input {
                        if code.language == 1 {
                            points_total += points_round;
                        } else {
                            lives -= 1;
                        }
                        break;
                    }
                    if let Key::Char('3') = input {
                        if code.language == 2 {
                            points_total += points_round;
                        } else {
                            lives -= 1;
                        }
                        break;
                    }
                    if let Key::Char('4') = input {
                        if code.language == 3 {
                            points_total += points_round;
                        } else {
                            lives -= 1;
                        }
                        break;
                    }
                }
            }
        }
        points_total
    };

    println!("Your total points {}!", points);
    Ok(())
}

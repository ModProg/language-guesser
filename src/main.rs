#![feature(try_blocks, duration_constants)]
use anyhow::Result;
use async_trait::async_trait;
use clap::ArgEnum;
use clap::Clap;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use rand::prelude::*;
use std::num::NonZeroU8;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, io};
use crossterm::{
    event::{self, Event, KeyCode as Key},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Wrap};
use tui::Terminal;

use crate::providers::github::GitHub;
use crate::providers::TestProvider;

mod providers;

#[derive(Debug)]
struct Code {
    reference: String,
    code: String,
    language: usize,
    options: Vec<String>,
}

#[async_trait]
trait CodeProvider: Send + Sync {
    async fn get_code(&self) -> Result<Code>;

    fn retries(&mut self, count: u8);
    fn options(&mut self, count: u8);
}

const MAX_POINTS: i32 = 12;
const STEP_DURATION: Duration = Duration::from_secs(2);
fn shown_chars(points: i32) -> i32 {
    2i32.pow((MAX_POINTS - points).max(0) as u32)
}

#[derive(ArgEnum)]
enum CodeProviders {
    #[clap(name = "github")]
    GitHub,
    #[clap(hidden(true))]
    Test,
}

#[derive(Clap)]
struct Options {
    /// How many options should be displayed when guessing the language
    #[clap(long, short, default_value = "4")]
    options: NonZeroU8,
    /// How often should webrequests be repeted on failure, only relevant for GitHub code
    /// provider
    #[clap(long, short, default_value = "8")]
    retries: NonZeroU8,
    /// What provider should be used for the code displayed
    ///
    /// * GitHub: pulls Code from a random repository licensed under MIT
    ///
    /// * BuiltIn: will use the code provided in ``
    #[clap(
        long,
        short,
        default_value = "Github",
        arg_enum,
        case_insensitive(true)
    )]
    provider: CodeProviders,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();

    let mut code_provider: Box<dyn CodeProvider> = match options.provider {
        CodeProviders::GitHub => {
            Box::new(GitHub::default().token(env::var("LANGUAGE_GUESSER_TOKEN").ok())?)
        }
        CodeProviders::Test => Box::new(TestProvider::default()),
    };
    code_provider.retries(options.retries.into());
    code_provider.options(options.options.into());
    let code_provider = Arc::new(code_provider);

    // Create Terminal
    let points = {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let rng = &mut rand::thread_rng();

        let mut points_total = 0;
        let mut lives = 5;
        let c = code_provider.clone();
        let mut next = Box::pin(tokio::spawn(async move { c.get_code().await }));
        'main: loop {
            if lives == 0 {
                break 'main;
            }
            let code = next.await??;
            let c = code_provider.clone();
            next = Box::pin(tokio::spawn(async move { c.get_code().await }));
            let language_descriptions = code.options.into_iter().zip(1..).collect::<Vec<_>>();
            let mut points_round = MAX_POINTS;
            let origin = rng.gen_range(0..code.code.len()) as i32;
            let text = code.code;
            let mut last = Instant::now();
            'tick: loop {
                if Instant::now().duration_since(last) > STEP_DURATION {
                    if points_round == 0 {
                        lives -= 1;
                        if lives == 0 {
                            break 'main;
                        } else {
                            break 'tick;
                        }
                    } else {
                        points_round -= 1;
                    }
                    last = Instant::now();
                }
                terminal.draw(|f| {
                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(6), Constraint::Percentage(80)].as_ref())
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
                                    Constraint::Ratio(1, 3),
                                    Constraint::Ratio(1, 3),
                                    Constraint::Ratio(1, 3),
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
                        .constraints([Constraint::Length(20), Constraint::Min(0)])
                        .split(vertical[1]);
                    let shown_chars = if points_round == 0 {
                        text.len() as i32
                    } else {
                        shown_chars(points_round)
                    };
                    let start = (text.len() as i32 - shown_chars)
                        .min(origin as i32 - shown_chars / 2)
                        .max(0);
                    let code = Paragraph::new(
                        text.as_str()
                            .chars()
                            .skip(start as usize)
                            .take(shown_chars as usize)
                            .collect::<String>(),
                    )
                    .wrap(Wrap { trim: false })
                    .block(Block::default().title("Code").borders(Borders::ALL));

                    f.render_widget(code, horizontal[1]);
                    let table =
                        Table::new(language_descriptions.iter().map(|(language, number)| {
                            Row::new(vec![number.to_string(), language.to_string()])
                        }))
                        .widths(&[Constraint::Length(3), Constraint::Percentage(100)])
                        .block(Block::default().title("Languages").borders(Borders::ALL));
                    f.render_widget(table, horizontal[0]);
                })?;

                if event::poll(Duration::ZERO)? {
                    if let Event::Key(KeyEvent {
                        code: key,
                        modifiers,
                    }) = event::read()?
                    {
                        if let (Key::Char('c'), KeyModifiers::CONTROL) = (key, modifiers) {
                            break 'main;
                        }
                        if let Key::Char('1') = key {
                            if code.language == 0 {
                                points_total += points_round;
                            } else {
                                lives -= 1;
                            }
                            break 'tick;
                        }
                        if let Key::Char('2') = key {
                            if code.language == 1 {
                                points_total += points_round;
                            } else {
                                lives -= 1;
                            }
                            break 'tick;
                        }
                        if let Key::Char('3') = key {
                            if code.language == 2 {
                                points_total += points_round;
                            } else {
                                lives -= 1;
                            }
                            break 'tick;
                        }
                        if let Key::Char('4') = key {
                            if code.language == 3 {
                                points_total += points_round;
                            } else {
                                lives -= 1;
                            }
                            break 'tick;
                        }
                    }
                }
            }
        }
        execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
        points_total
    };

    println!("\n\nYour total points {}!\n\n", points);
    Ok(())
}

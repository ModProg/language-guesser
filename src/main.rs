use anathema::display::disable_raw_mode;
use anathema::runtime::Event;
use anathema::runtime::Runtime;
use anathema::templates::DataCtx;
use anyhow::{bail, Result};
use async_trait::async_trait;
use clap::ArgEnum;
use clap::Parser;
use crossterm::event::KeyCode;
use crossterm::execute;
use futures::select;
use futures::FutureExt;
use rand::prelude::*;
use std::io::stdout;
use std::num::NonZeroU8;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::{env, io};

use crate::providers::github::GitHub;
use crate::providers::TestProvider;

mod providers;
mod util;

#[derive(Debug, Clone)]
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

const MAX_POINTS: i64 = 12;
const MAX_LIVES: i64 = 5;
const STEP_DURATION: Duration = Duration::from_secs(2);
fn shown_chars(points: i64) -> usize {
    2usize.pow((MAX_POINTS - points).max(0) as u32)
}

#[derive(ArgEnum, Clone)]
enum CodeProviders {
    #[clap(name = "github")]
    GitHub,
    #[clap(hide = true)]
    Test,
}

#[derive(Parser)]
struct Options {
    /// How many options should be displayed when guessing the language
    /// This should be at least 2 and at most 10
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
        default_value = "github",
        arg_enum,
        case_insensitive(true)
    )]
    provider: CodeProviders,
    /// An optional list of language to use. If omitted, all languages on github will be used.
    #[clap(long, short)]
    languages: Vec<String>,
}

enum UserEvent {
    Code(String),
    TotalPoints(i64),
    RoundPoints(i64),
    Lives(u64),
    // TODO make usize
    Languages(Vec<(u64, String)>),
}

enum InputEvent {
    Guess(usize),
    Quit,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();

    if !options.languages.is_empty() && options.languages.len() < options.options.get() as usize {
        bail!("Not enough languages! Need at least {}", options.options);
    }

    let mut code_provider: Box<dyn CodeProvider> = match options.provider {
        CodeProviders::GitHub => Box::new(
            GitHub::new(options.languages)
                .await?
                .token(env::var("LANGUAGE_GUESSER_TOKEN").ok())?,
        ),
        CodeProviders::Test => Box::new(TestProvider::default()),
    };
    code_provider.retries(options.retries.into());
    code_provider.options(options.options.into());
    let code_provider = Arc::new(code_provider);

    // // Create Terminal
    // let (points, codes) = {
    //     enable_raw_mode()?;
    //     let mut stdout = io::stdout();
    //     let backend = CrosstermBackend::new(stdout);
    //     let mut terminal = Terminal::new(backend)?;
    //     terminal.clear()?;
    //
    //     let rng = &mut rand::thread_rng();
    //
    //     let mut points_total = 0;
    //     let mut codes: Vec<(Code, Option<i32>)> = Vec::new();
    //     let mut lives = 5;
    //     let c = code_provider.clone();
    //     let mut next = Box::pin(tokio::spawn(async move { c.get_code().await }));
    //     'main: loop {
    //         if lives == 0 {
    //             break 'main;
    //         }
    //         let code = next.await??;
    //         let c = code_provider.clone();
    //         next = Box::pin(tokio::spawn(async move { c.get_code().await }));
    //         let language_descriptions = code
    //             .options
    //             .clone()
    //             .into_iter()
    //             .zip(1..)
    //             .collect::<Vec<_>>();
    //         let mut points_round = MAX_POINTS;
    //         let origin = loop {
    //             let origin = rng.gen_range(0..code.code.len());
    //             if !code.code.chars().nth(origin).unwrap().is_whitespace() {
    //                 break origin as i32;
    //             };
    //         };
    //         let text = code.code.clone();
    //         let mut last = Instant::now();
    //         'tick: loop {
    //             if Instant::now().duration_since(last) > STEP_DURATION {
    //                 if points_round == 0 {
    //                     lives -= 1;
    //                     if lives == 0 {
    //                         break 'main;
    //                     } else {
    //                         break 'tick;
    //                     }
    //                 } else {
    //                     points_round -= 1;
    //                 }
    //                 last = Instant::now();
    //             }
    //             terminal.draw(|f| {
    //                 let vertical = Layout::default()
    //                     .direction(Direction::Vertical)
    //                     .constraints([Constraint::Length(6), Constraint::Percentage(80)].as_ref())
    //                     .split(f.size());
    //                 let block = Block::default().borders(Borders::ALL);
    //                 {
    //                     let inner = Layout::default()
    //                         .direction(Direction::Vertical)
    //                         .constraints(
    //                             [
    //                                 Constraint::Length(2),
    //                                 Constraint::Length(1),
    //                                 Constraint::Percentage(100),
    //                             ]
    //                             .as_ref(),
    //                         )
    //                         .split(block.inner(vertical[0]));
    //                     let paragraph = Paragraph::new(
    //                         "Press CTRL+C if you want to give up.\nPress 1-4 to guess a language.",
    //                     );
    //                     f.render_widget(paragraph, inner[0]);
    //                     let bottom = Layout::default()
    //                         .direction(Direction::Horizontal)
    //                         .constraints(
    //                             [
    //                                 Constraint::Ratio(1, 3),
    //                                 Constraint::Ratio(1, 3),
    //                                 Constraint::Ratio(1, 3),
    //                             ]
    //                             .as_ref(),
    //                         )
    //                         .split(inner[2]);
    //                     let p = Paragraph::new(format!("Total Points: {}", points_total));
    //                     f.render_widget(p, bottom[0]);
    //
    //                     let p = Paragraph::new(format!("Round Points: {}", points_round));
    //                     f.render_widget(p, bottom[1]);
    //
    //                     let p = Paragraph::new(format!("Lives: {}", "🫀".repeat(lives)));
    //                     f.render_widget(p, bottom[2]);
    //                 }
    //                 f.render_widget(block, vertical[0]);
    //
    //                 let horizontal = Layout::default()
    //                     .direction(Direction::Horizontal)
    //                     .constraints([Constraint::Length(20), Constraint::Min(0)])
    //                     .split(vertical[1]);
    //                 let shown_chars = if points_round == 0 {
    //                     text.len() as i32
    //                 } else {
    //                     shown_chars(points_round)
    //                 };
    //                 let start = (text.len() as i32 - shown_chars)
    //                     .min(origin as i32 - shown_chars / 2)
    //                     .max(0);
    //                 let code = Paragraph::new(
    //                     text.as_str()
    //                         .chars()
    //                         .skip(start as usize)
    //                         .take(shown_chars as usize)
    //                         .collect::<String>(),
    //                 )
    //                 .wrap(Wrap { trim: false })
    //                 .block(Block::default().title("Code").borders(Borders::ALL));
    //
    //                 f.render_widget(code, horizontal[1]);
    //                 let table =
    //                     Table::new(language_descriptions.iter().map(|(language, number)| {
    //                         Row::new(vec![number.to_string(), language.to_string()])
    //                     }))
    //                     .widths(&[Constraint::Length(3), Constraint::Percentage(100)])
    //                     .block(Block::default().title("Languages").borders(Borders::ALL));
    //                 f.render_widget(table, horizontal[0]);
    //             })?;
    //
    //             if event::poll(Duration::ZERO)? {
    //                 if let Event::Key(KeyEvent {
    //                     code: key,
    //                     modifiers,
    //                 }) = event::read()?
    //                 {
    //                     if let (Key::Char('c'), KeyModifiers::CONTROL) = (key, modifiers) {
    //                         break 'main;
    //                     }
    //                     if let Key::Char('q') = key {
    //                         break 'main;
    //                     }
    //                     if let Key::Char(char) = key {
    //                         if ('1'..='9').contains(&char) {
    //                             if char as usize - '1' as usize == code.language {
    //                                 points_total += points_round;
    //                                 codes.push((code, Some(points_round)));
    //                             } else {
    //                                 lives -= 1;
    //                                 codes.push((code, None));
    //                             }
    //                             break 'tick;
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     disable_raw_mode()?;
    //     (points_total, codes)
    // };
    //
    const TEMPLATE: &str = include_str!("ui.tiny");
    let mut runtime = Runtime::<UserEvent>::new();
    // runtime.output_cfg.alt_screen = true;
    let runtime_sender = runtime.sender();
    let (tx, rx) = flume::unbounded::<InputEvent>();

    let mut data = DataCtx::empty();
    data.set("total_points", 0i64);
    data.set("round_points", MAX_POINTS);
    data.set("lives", MAX_LIVES);
    let ui = thread::spawn(move || {
        runtime
            .start(TEMPLATE, data, |event, _ctx, ctx, runtime_tx| {
                if event.ctrl_c() || matches!(event.get_keycode(), Some(KeyCode::Char('q'))) {
                    let _ = runtime_tx.send(Event::Quit);
                    let _ = tx.send(InputEvent::Quit);
                }
                if let Some(KeyCode::Char(c)) = event.get_keycode() {
                    if let Some(digit) = c.to_digit(10) {
                        let _ = tx.send(InputEvent::Guess(digit as usize));
                    }
                }

                if let Some(event) = event.user() {
                    match event {
                        UserEvent::Code(code) => ctx.set("code", code),
                        UserEvent::TotalPoints(points) => ctx.set("total_points", points),
                        UserEvent::RoundPoints(points) => ctx.set("round_points", points),
                        UserEvent::Lives(lives) => ctx.set("lives", lives),
                        UserEvent::Languages(languages) => ctx.set("languages", languages),
                    }
                }
            })
            .expect("Hardcoded template is valid");
    });

    let mut points_round = MAX_POINTS;
    let mut points_total = 0;
    let mut codes: Vec<(Code, Option<i64>)> = Vec::new();
    let mut lives = 5;

    let mut event = rx.recv_async().fuse();
    let mut code_req = code_provider.get_code().fuse();

    let mut current_code: Option<Code> = None;

    loop {
        select! {
            event_ = event => {
                match event_? {
                    InputEvent::Guess(language) => {
                        if let Some(code) =current_code.take() {
                            if code.language == language {
                                points_total += points_round;
                                runtime_sender.send(Event::User(UserEvent::TotalPoints(points_total)))?;
                                codes.push((code, Some(points_total)))
                            } else {
                                codes.push((code, None));
                            }

                            code_req = code_provider.get_code().fuse();

                            runtime_sender.send(Event::User(UserEvent::RoundPoints(points_round)))?;
                        }
                    },
                    InputEvent::Quit => break,
                }
                event = rx.recv_async().fuse();
            },
            code_ = code_req => {
                let code = code_?;
                runtime_sender.send(Event::User(UserEvent::Code(code.code.clone())))?;
                runtime_sender.send(Event::User(UserEvent::Languages(code.options.iter().cloned().enumerate().map(|(i,l)|(i as u64, l)).collect())))?;
                current_code = Some(code);
            }
        };
    }
    ui.join().expect("UI thread exits successfully");
    println!("\nYour total points {}!\n\nDetails:", points_total);
    {
        use comfy_table::modifiers::UTF8_SOLID_INNER_BORDERS;
        use comfy_table::presets::UTF8_FULL;
        use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(
                vec!["Score", "Language", "Reference"]
                    .iter()
                    .map(|s| Cell::new(s).set_alignment(CellAlignment::Center)),
            );
        for (code, points) in codes {
            table.add_row(vec![
                Cell::new(
                    points
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| String::from("---")),
                ),
                Cell::new(code.options[code.language].clone()),
                Cell::new(code.reference),
            ]);
        }
        println!("{}", table);
    }
    Ok(())
}

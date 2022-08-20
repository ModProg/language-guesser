use std::{env, num::NonZeroU8, sync::Arc, thread, time::Duration};

use anathema::{
    runtime::{
        Event::{self, User},
        Runtime,
    },
    templates::DataCtx,
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use clap::{ArgEnum, Parser};
use crossterm::event::KeyCode;
use futures::{select, FutureExt};
use rand::prelude::*;
use tokio::time::MissedTickBehavior;

use crate::providers::{github::GitHub, TestProvider};

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

    const TEMPLATE: &str = include_str!("ui.tiny");
    let mut runtime = Runtime::<UserEvent>::new();
    runtime.output_cfg.alt_screen = true;
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
    let mut tick = tokio::time::interval(STEP_DURATION);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    // let mut tick = tick..fuse();

    // TODO preload code
    let mut current_code: Option<Code> = None;
    let mut origin = 0usize;

    fn code_section(code: &str, origin: usize, points: i64) -> String {
        let shown_chars = if points == 0 {
            code.len()
        } else {
            shown_chars(points).min(code.len())
        };
        let start = (code.len() - shown_chars).min(origin.saturating_sub(shown_chars / 2));
        code.chars()
            .skip(start as usize)
            .take(shown_chars as usize)
            .collect()
    }

    loop {
        select! {
            _ = tick.tick().fuse() => {
                if let Some(code) = &current_code {
                    if points_round == 0 {
                        code_req = code_provider.get_code().fuse();
                        points_round = MAX_POINTS;
                        runtime_sender.send(User(UserEvent::Code("".to_string())))?;
                        runtime_sender.send(User(UserEvent::RoundPoints(points_round)))?;
                        codes.push((current_code.take().expect("There is a code"), None));
                        if lives == 1 {
                            break
                        } else{
                            lives -=1;
                            runtime_sender.send(User(UserEvent::Lives(lives)))?;
                        }
                    } else {
                        points_round -= 1;
                        runtime_sender.send(User(UserEvent::Code(code_section(&code.code, origin, points_round))))?;
                        runtime_sender.send(User(UserEvent::RoundPoints(points_round)))?;
                    }
                }
            },
            event_ = event => {
                match event_? {
                    InputEvent::Guess(language) => {
                        if let Some(code) =current_code.take() {
                            if code.language == language {
                                points_total += points_round;
                                points_round = MAX_POINTS;
                                runtime_sender.send(Event::User(UserEvent::TotalPoints(points_total)))?;
                                codes.push((code, Some(points_total)))
                            } else {
                                points_round = MAX_POINTS;
                                codes.push((code, None));
                                if lives == 1 {
                                    break
                                } else{
                                    lives -=1;
                                    runtime_sender.send(User(UserEvent::Lives(lives)))?;
                                }
                            }

                            code_req = code_provider.get_code().fuse();

                            runtime_sender.send(Event::User(UserEvent::RoundPoints(points_round)))?;
                            runtime_sender.send(Event::User(UserEvent::Code("".to_string())))?;
                        }
                    },
                    InputEvent::Quit => break,
                }
                event = rx.recv_async().fuse();
            },
            code_ = code_req => {
                tick.reset();
                let code = code_?;
                runtime_sender.send(Event::User(UserEvent::Languages(code.options.iter().cloned().enumerate().map(|(i,l)|(i as u64, l)).collect())))?;
                origin = loop {
                    let origin = thread_rng().gen_range(0..code.code.len());
                    if !code.code.chars().nth(origin).unwrap().is_whitespace() {
                        break origin;
                    };
                };
                runtime_sender.send(User(UserEvent::Code(code_section(&code.code, origin, points_round))))?;
                current_code = Some(code);
            }
        };
    }
    ui.join().expect("UI thread exits successfully");
    println!("\nYour total points {}!\n\nDetails:", points_total);
    {
        use comfy_table::{
            modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, Cell, CellAlignment,
            ContentArrangement, Table,
        };
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

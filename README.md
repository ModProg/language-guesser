# LanguageGuesser

The goal is simple: **Guess the programming language shown**.

You are given 5 lives and loose one every time you cannot guess the language quickly enough.

You start every round with 12 points, this number decreases every 2 seconds. If you do not guess until it hits 0 you will loose a live.

If you guess wrongly you will loose a live.

With every point decrease more characers are displayed, so waiting may help you but reduces your score.

## Token (Optional)

The code is from GitHub (all MIT licensed) due to API limitations it is recommended to set a `Personal access token`: <https://github.com/settings/tokens>, you don't need to allow any scopes as this is only to lift the IP-Ratelimit.

If you have a PAT you can provide it via the environment variable `LANGUAGE_GUESSER_TOKEN`.

Don't worry you can play without that, but bear in mind that you will only be play 60 "files" per hour as that is the GitHub rate limit.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│Press CTRL+C if you want to give up.                                        │
│Press 1-4 to guess a language.                                              │
│                                                                            │
│Total Points: 0          Round Points: 2          Lives: ❦ ❦ ❦ ❦ ❦          │
└────────────────────────────────────────────────────────────────────────────┘
┌Languages─────────┐┌Code────────────────────────────────────────────────────┐
│1   javascript    ││nhandled_error = false                                  │
│2   ruby          ││                                                        │
│3   typescript    ││Rails.application.configure do                          │
│4   java          ││  config.active_job.queue_adapter = :good_job           │
│                  ││  config.good_job.execution_mode = :async               │
│                  ││  config.good_job.poll_interval = 30                    │
│                  ││                                                        │
│                  ││  config.good_job.enable_cron = true                    │
│                  ││  config.good_job.cron = {                              │
│                  ││    frequent_example: {                                 │
│                  ││      description: "Enqueue an ExampleJob with a random │
│                  ││sample of configuration",                               │
│                  ││      cron: "*/5 * * * * *", # every 5 seconds          │
│                  ││      class: "ExampleJob",                              │
│                  ││      args: [],                                         │
│                  ││      set: (lambda do                                   │
└──────────────────┘└────────────────────────────────────────────────────────┘
```
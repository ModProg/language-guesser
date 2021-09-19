# LanguageGuesser

The goal is simple: **Guess the programming language shown**.

You are given 5 lives and loose one every time you cannot guess the language quickly enough.

You start every round with 12 points, this number decreases every 2 seconds. If you do not guess until it hits 0 you will loose a live.

If you guess wrongly you will loose a live.

With every point decrease more characers are displayed, so waiting may help you but reduces your score.

My Highscore was 34 (at 5am).

## Token (Optional)

The code is from GitHub (all MIT licensed) due to API limitations it is recommended to set a `Personal access token`: <https://github.com/settings/tokens>, you don't need to allow any scopes as this is only to lift the IP-Ratelimit.

If you have a PAT you can provide it via the environment variable `LANGUAGE_GUESSER_TOKEN`.

Don't worry you can play without that, but bear in mind that you will only be able to play 60 "files" per hour as that is the GitHub rate limit.

## "Screenshots"
```text
┌─────────────────────────────────────────────────────────────────────────┐
│Press CTRL+C if you want to give up.                                     │
│Press 1-4 to guess a language.                                           │
│                                                                         │
│Total Points: 0         Round Points: 1         Lives: 🫀🫀🫀🫀          │
└─────────────────────────────────────────────────────────────────────────┘
┌Languages─────────┐┌Code─────────────────────────────────────────────────┐
│1   ruby          ││---@class cmp.Cache                                  │
│2   c#            ││---@field public entries any                         │
│3   lua           ││local cache = {}                                     │
│4   css           ││                                                     │
│                  ││cache.new = function()                               │
│                  ││  local self = setmetatable({}, { __index = cache }) │
│                  ││  self.entries = {}                                  │
│                  ││  return self                                        │
│                  ││end                                                  │
│                  ││                                                     │
│                  ││---Get cache value                                   │
│                  ││---@param key string                                 │
│                  ││---@return any|nil                                   │
│                  ││cache.get = function(self, key)                      │
│                  ││  key = self:key(key)                                │
│                  ││  if self.entries[key] ~= nil then                   │
│                  ││    return unpack(self.entries[key])                 │
│                  ││  end                                                │
│                  ││  return nil                                         │
│                  ││end                                                  │
└──────────────────┘└─────────────────────────────────────────────────────┘
```
```text
Your total points 7!

Details:
┌───────┬──────────┬──────────────────────────────────────────────────────┐
│ Score │ Language │                       Reference                      │
╞═══════╪══════════╪══════════════════════════════════════════════════════╡
│ 7     │ css      │ https://github.com/getferdi/recipes/blob/cd082f103b8 │
│       │          │ b401ca23dc89aaa3c15f6f077c889/recipes/google-transla │
│       │          │ te/service.css                                       │
├───────┼──────────┼──────────────────────────────────────────────────────┤
│ ---   │ ruby     │ https://github.com/timdorr/tesla-api/blob/8a2accea08 │
│       │          │ 3e0ba4554040197779b4454ba39085/Rakefile              │
├───────┼──────────┼──────────────────────────────────────────────────────┤
│ ---   │ c        │ https://github.com/gbdk-2020/gbdk-2020/blob/efba0aff │
│       │          │ 9bf8fd61be1c76d364f9a3e65691154b/gbdk-support/lcc/lc │
│       │          │ c.c                                                  │
├───────┼──────────┼──────────────────────────────────────────────────────┤
│ ---   │ shell    │ https://github.com/EnergizedProtection/block/blob/c5 │
│       │          │ ef324ed8ef1e466276a6a8c837d7c5f0f2f39b/bluGo/build.s │
│       │          │ h                                                    │
└───────┴──────────┴──────────────────────────────────────────────────────┘
```

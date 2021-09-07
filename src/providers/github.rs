use crate::{Code, CodeProvider};
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use octocrab::Octocrab;
use rand::{prelude::*, thread_rng};
use serde::Deserialize;

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
    "lisp",
    "shell",
    "vim",
    "lua",
];

#[derive(Deserialize, Debug)]
struct CodeRequest {
    download_url: String,
}

#[derive(Clone, Copy)]
pub struct GitHub {
    language_count: usize,
    retries: u8,
}

impl Default for GitHub {
    fn default() -> Self {
        GitHub {
            language_count: 4,
            retries: 8,
        }
    }
}

impl GitHub {
    pub fn token(self, token: Option<String>) -> Result<Self> {
        if let Some(token) = token {
            octocrab::initialise(Octocrab::builder().personal_token(token))?;
        }
        Ok(self)
    }
}

#[async_trait]
impl CodeProvider for GitHub {
    async fn get_code(&self) -> Result<Code> {
        // return Ok(Code {
        //     reference: "".to_string(),
        //     code: " ".to_string(),
        //     language: 0,
        //     options: vec!["".to_string()],
        // });
        let octocrab = octocrab::instance();
        for _ in 0..self.retries {
            let _: Result<()> = try {
                let languages: Vec<String> = LANGUAGES
                    .choose_multiple(&mut thread_rng(), self.language_count)
                    .map(|s| s.to_string())
                    .collect();
                let idx = thread_rng().gen_range(0..languages.len());
                let language = &languages[idx];

                let repos = octocrab
                    .search()
                    .repositories(&format!("language:{} license:mit stars:>=30", language))
                    .sort("updated")
                    .send()
                    .await?
                    .items;

                let repo = repos.choose(&mut thread_rng()).ok_or_else(|| anyhow!(""))?;

                let files = octocrab
                    .search()
                    .code(&format!("language:{} repo:{}", language, repo.full_name))
                    .send()
                    .await?
                    .items;

                let file = files.choose(&mut thread_rng()).ok_or_else(|| anyhow!(""))?;

                let code: CodeRequest = octocrab.get(&file.url, None::<&()>).await?;
                let code: String = octocrab
                    ._get(code.download_url, None::<&()>)
                    .await?
                    .text()
                    .await?;
                return Ok(Code {
                    reference: repo.full_name.clone(),
                    code: code.max(" ".to_string()),
                    language: idx,
                    options: languages,
                });
            };
        }
        bail!("Unable to get new code from GitHub.");
    }

    fn retries(&mut self, count: u8) {
        self.retries = count;
    }

    fn options(&mut self, count: u8) {
        self.language_count = count.into();
    }
}

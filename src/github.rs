use crate::{Code, CodeProvider};
use anyhow::{anyhow, Result};
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

pub struct GitHub {
    language_count: usize,
}

impl Default for GitHub {
    fn default() -> Self {
        GitHub { language_count: 4 }
    }
}

impl GitHub {
    pub fn token(self, token: Option<String>) -> Result<Self> {
        if let Some(token) = token {
            octocrab::initialise(Octocrab::builder().personal_token(token))?;
        }
        Ok(self)
    }

    pub fn language_count(mut self, language_count: usize) -> Self {
        self.language_count = language_count;
        self
    }
}

#[async_trait]
impl CodeProvider for GitHub {
    async fn get_code(&self) -> Result<Code> {
        let octocrab = octocrab::instance();
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
        Ok(Code {
            repository: repo.clone(),
            code,
            language: idx,
            options: languages,
        })
    }
}

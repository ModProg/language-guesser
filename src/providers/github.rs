use crate::{util::DeserializeKeys, Code, CodeProvider};
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use octocrab::Octocrab;
use rand::{prelude::*, thread_rng};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CodeRequest {
    download_url: String,
}

#[derive(Clone)]
pub struct GitHub {
    language_count: usize,
    retries: u8,
    languages: Vec<String>,
}

impl GitHub {
    pub async fn new() -> Result<Self> {
        let res = octocrab::instance()
            ._get(
                "https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml",
                None::<&()>,
            )
            .await?
            .bytes()
            .await?;

        let DeserializeKeys(languages) = serde_yaml::from_slice(&res)?;

        Ok(GitHub {
            language_count: 4,
            retries: 8,
            languages,
        })
    }

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
                let languages: Vec<String> = self
                    .languages
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
                    reference: file.html_url.to_string(),
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

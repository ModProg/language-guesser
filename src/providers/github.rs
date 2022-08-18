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
    pub async fn new(mut languages: Vec<String>) -> Result<Self> {
        if languages.is_empty() {
            let res = octocrab::instance()
                ._get(
                    "https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml",
                    None::<&()>,
                )
                .await?
                .bytes()
                .await?;

            let DeserializeKeys(langs) = serde_yaml::from_slice(&res)?;

            languages = langs;
        }

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

macro_rules! continue_on_error {
    ($($tts:tt)*) => {
        match $($tts)* {
            Ok(v) => v,
            _ => continue,
        }
    };
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
            let languages: Vec<String> = self
                .languages
                .choose_multiple(&mut thread_rng(), self.language_count)
                .map(|s| s.to_string())
                .collect();
            let idx = thread_rng().gen_range(0..languages.len());
            let language = &languages[idx];

            let repos = continue_on_error!(
                octocrab
                    .search()
                    .repositories(&format!("language:{} license:mit stars:>=30", language))
                    .sort("updated")
                    .send()
                    .await
            )
            .items;

            let repo =
                continue_on_error!(repos.choose(&mut thread_rng()).ok_or_else(|| anyhow!("")));

            let files = continue_on_error!(
                octocrab
                    .search()
                    .code(&format!(
                        "language:{} repo:{}",
                        language,
                        repo.full_name.as_deref().unwrap_or_default()
                    ))
                    .send()
                    .await
            )
            .items;

            let file =
                continue_on_error!(files.choose(&mut thread_rng()).ok_or_else(|| anyhow!("")));

            let code: CodeRequest = continue_on_error!(octocrab.get(&file.url, None::<&()>).await);
            let code: String = continue_on_error!(
                continue_on_error!(octocrab._get(code.download_url, None::<&()>).await)
                    .text()
                    .await
            );
            return Ok(Code {
                reference: file.html_url.to_string(),
                code: code.max(" ".to_string()),
                language: idx,
                options: languages,
            });
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

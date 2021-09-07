use anyhow::Result;
use async_trait::async_trait;

use crate::{Code, CodeProvider};

pub mod github;

#[derive(Default)]
pub struct TestProvider {
    options: usize,
}

#[async_trait]
impl CodeProvider for TestProvider {
    async fn get_code(&self) -> Result<Code> {
        Ok(Code {
            reference: "test".into(),
            code: "ABC".into(),
            language: 0,
            options: vec!["a", "b", "c", "d"]
                .into_iter()
                .take(self.options)
                .map(&str::to_string)
                .collect(),
        })
    }

    fn retries(&mut self, _count: u8) {}

    fn options(&mut self, count: u8) {
        self.options = count.into();
    }
}

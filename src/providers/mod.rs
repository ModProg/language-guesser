use anyhow::Result;
use async_trait::async_trait;
use rand::{thread_rng, Rng};

use crate::{Code, CodeProvider};

pub mod github;

#[derive(Default)]
pub struct TestProvider {
    options: usize,
}

#[async_trait]
impl CodeProvider for TestProvider {
    async fn get_code(&self) -> Result<Code> {
        let c: char = thread_rng().gen_range('a'..'d');
        Ok(Code {
            reference: "test".into(),
            code: c.to_string().repeat(20),
            language: c as usize - 'a' as usize,
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

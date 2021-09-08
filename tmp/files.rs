use crate::CodeProvider;

pub struct FileProvider {
    options: u8,
    languages: Vec<String>,
    files: HashMap<String, Vec<String>>,
}

impl FileProvider {
    fn new() -> FileProvider {

    }
}

impl CodeProvider for FileProvider {
    async fn get_code(&self) -> anyhow::Result<crate::Code> {
        todo!()
    }

    fn retries(&mut self, _: u8) {}

    fn options(&mut self, count: u8) {
        self.options = count;
    }
}

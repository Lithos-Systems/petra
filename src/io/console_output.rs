use crate::tags::TagManager;

pub struct ConsoleOutput {
    pub tag: &'static str,
}

impl ConsoleOutput {
    pub fn new(tag: &'static str) -> Self { Self { tag } }
    pub async fn execute(&self, tags: &TagManager) {
        if let Some(val) = tags.read(self.tag).await {
            println!("{} = {:?}", self.tag, val);
        }
    }
}

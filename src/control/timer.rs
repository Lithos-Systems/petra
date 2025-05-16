use tokio::time::{Instant, Duration};

#[derive(Clone)]
pub struct Ton {
    pub in_: bool,
    pub pt: Duration,
    pub q: bool,
    pub start_time: Option<Instant>,
}

impl Ton {
    pub fn new(pt_ms: u64) -> Self {
        Self { in_: false, pt: Duration::from_millis(pt_ms), q: false, start_time: None }
    }
    pub async fn execute(&mut self) {
        if self.in_ {
            if self.start_time.is_none() { self.start_time = Some(Instant::now()); }
            if self.start_time.unwrap().elapsed() >= self.pt { self.q = true; }
        } else {
            self.q = false;
            self.start_time = None;
        }
    }
}

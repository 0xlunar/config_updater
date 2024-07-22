use std::fs::File;
use std::sync::Arc;
use std::io::BufReader;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use log::info;
use serde::de::DeserializeOwned;
use tokio::time::{Instant, sleep_until};

pub struct ConfigMonitor<T>{
    filename: String,
    recheck_delay_seconds: u64,
    data: Arc<Mutex<T>>,
}

impl<T: DeserializeOwned + Send + 'static> ConfigMonitor<T> {
    pub fn new(filename: &str, recheck_delay_seconds: Option<u64>) -> Self {
        let data = Self::load_file(filename);

        Self {
            filename: filename.to_string(),
            recheck_delay_seconds: recheck_delay_seconds.unwrap_or(300),
            data: Arc::new(Mutex::new(data))
        }
    }

    pub fn data(&self) -> Arc<Mutex<T>> {
        self.data.clone()
    }

    pub fn monitor(self) -> JoinHandle<()> {
        let config_data = Arc::clone(&self.data);
        tokio::task::spawn(async move {
            let mut file_last_modified = self.file_last_modified();

            loop {
                let file_recent_modified = self.file_last_modified();

                if file_last_modified != file_recent_modified {
                    info!("Found file changes, updating config...");
                    file_last_modified = file_recent_modified;
                    let data = Self::load_file(&self.filename);
                    let mut lock = config_data.lock().await;
                    *lock = data;
                }

                sleep_until(Instant::now() + Duration::from_secs(self.recheck_delay_seconds)).await
            }
        })
    }

    fn load_file(filename: &str) -> T {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }

    fn file_last_modified(&self) -> u64 {
        let file = File::open(&self.filename).unwrap();
        file
            .metadata()
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

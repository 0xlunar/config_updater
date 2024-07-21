use std::fs::File;
use std::sync::Arc;
use std::io::{BufReader, Read};
use std::time::{Duration, UNIX_EPOCH};
use serde::Deserialize;
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
    pub fn new<S: AsRef<str>>(filename: S, recheck_delay_seconds: Option<u64>) -> Self {
        let data = Self::load_file(filename.as_ref());

        Self {
            filename: filename.as_ref().to_string(),
            recheck_delay_seconds: recheck_delay_seconds.unwrap_or(300),
            data: Arc::new(Mutex::new(data))
        }
    }

    pub fn data(&self) -> Arc<Mutex<T>> {
        self.data.clone()
    }

    pub fn monitor(self) -> JoinHandle<()> {
        let c_config = Arc::clone(&self.data);
        tokio::task::spawn(async move {
            let file = File::open(&self.filename).unwrap();
            let mut file_last_modified = file
                .metadata()
                .unwrap()
                .modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            loop {
                let file = File::open(&self.filename).unwrap();
                let file_recent_modified = file
                    .metadata()
                    .unwrap()
                    .modified()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if file_last_modified != file_recent_modified {
                    info!("Found file changes, updating config");
                    file_last_modified = file_recent_modified;
                    let mut lock = c_config.lock().await;
                    let data = Self::load_file(&self.filename);
                    *lock = data;
                }
                sleep_until(Instant::now() + Duration::from_secs(self.recheck_delay_seconds)).await
            }
        })
    }

    fn load_file<S: AsRef<str>>(filename: S) -> T {
        let file = File::open(filename.as_ref()).unwrap();
        let mut reader = BufReader::new(file);
        // let mut buffer = String::new();
        // reader.read_to_string(&mut buffer);
        serde_json::from_reader(reader).unwrap()
    }
}

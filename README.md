# config_updater
Easy to use configuration updater.

Automatically update your config when changes are made instead of restarting each time.

## Example
```rust
use serde::Deserialize;
use config_updater::ConfigMonitor;

#[derive(Deserialize)]
struct MyConfig {
    id: u64,
}

#[tokio::main]
async fn main() {
    let config_monitor: ConfigMonitor<MyConfig> = ConfigMonitor::new("./config.json", Some(30));
    let my_config = config_monitor.data(); // Arc<Mutex<MyConfig>>
    let config_handle = config_monitor.monitor();
    
    let c_my_config = my_config.clone();
    tokio::spawn(async {
        // Do Something with c_my_config
        let my_id = {
            let lock = c_my_config.lock().await;
            lock.id.clone();
        };
        println!("My ID: {}", my_id);
    });
    
    config_handle.await.unwrap();
}
```

use config::Config;
use fetch::{fetch_data, prepare_hosts};
use data::Data;

use std::time::Instant;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Server {
    config: Mutex<Config>,
    data: RwLock<Data>,
    running: AtomicBool,
    warnings_sent: Mutex<HashMap<(String, String), Instant>>,
}

/// Handles concurrent access to config/data.
///
/// All public functions take a `&self` and handle mutability internally.
///
/// This makes this class safe to put inside an `Arc`.
impl Server {
    /// Creates a new Server, and starts a separate thread to periodically refresh the data.
    pub fn new(config: Config) -> Arc<Self> {
        // Prepare the hosts the first time

        let result = Arc::new(Server {
            config: Mutex::new(config),
            data: RwLock::new(Data::default()),
            // Indicate that the refresh thread is running
            running: AtomicBool::new(true),
            warnings_sent: Mutex::new(HashMap::new()),
        });

        // Spawn a refresh thread.
        let cloned = result.clone();
        thread::spawn(move || {
            // Refresh once immediately
            prepare_hosts(&cloned.current_conf());
            cloned.refresh();

            // TODO: select! on a refresh channel and a timer.
            while cloned.running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(cloned.refresh_delay()));
                cloned.refresh();
            }
        });

        result
    }

    fn refresh_delay(&self) -> u64 {
        self.current_conf().refresh_delay.unwrap_or(30)
    }

    /// Update the configuration, and refresh everything.
    ///
    /// You should run this async (in a thread::spawn for instance).
    pub fn with_conf<E, F>(&self, update: F) -> Result<(), E>
    where
        F: FnOnce(&mut Config) -> Result<(), E>,
    {
        {
            let mut config = self.current_conf();
            update(&mut config)?;
            prepare_hosts(&config);
        }

        // And then just refresh
        // TODO: send a signal to the refresh thread instead?
        self.refresh();
        Ok(())
    }

    pub fn current_conf(&self) -> MutexGuard<Config> {
        self.config.lock().unwrap()
    }

    /// Get a read access to the latest data.
    pub fn latest_data(&self) -> RwLockReadGuard<Data> {
        self.data.read().unwrap()
    }

    pub fn refresh(&self) {
        // Don't lock while we're fetching.
        println!("Refreshing.");
        let start = Instant::now();
        let conf = self.current_conf().clone();
        let fresh = fetch_data(&conf);
        let mut warnings_sent = self.warnings_sent.lock().unwrap();
        if let Some(ref slack) = conf.slack {
            for host in &fresh.hosts {
                for disk in &host.disks {
                    if let (Some(used), Some(size)) = (disk.used, disk.size) {
                        if (100 * used) > (size * 98) {
                            let hostname = host.hostname
                                .as_ref()
                                .map(String::as_str)
                                .unwrap_or("");
                            let key =
                                (hostname.to_string(), disk.name.clone());
                            if let Some(last) = warnings_sent.get(&key) {
                                if last.elapsed()
                                    < Duration::from_secs(60 * 30)
                                {
                                    continue;
                                }
                            }
                            warnings_sent.insert(key, Instant::now());
                            if let Err(err) = ::slack::send_alert(
                                &slack.hook,
                                &slack.channel,
                                hostname,
                                &disk.name,
                                &disk.mountpoint,
                                (100 * used) / size,
                            ) {
                                println!(
                                    "Error sending slack notification: {}",
                                    err
                                );
                            }
                        }
                    }
                }
            }
        }

        let mut data = self.data.write().unwrap();
        *data = fresh;
        println!("Refreshed ({:?})", start.elapsed());
    }

    /// Stops the refresh thread.
    ///
    /// This is called automatically on drop.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}

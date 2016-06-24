use config::Config;
use fetch::{fetch_data, prepare_hosts, Data};

use std::sync;
use std::thread;
use std::time::Duration;

pub struct Server {
    config: sync::Mutex<Config>,
    data: sync::RwLock<Option<Data>>,
    running: sync::Mutex<bool>,
}

/// Handles concurrent access to config/data.
///
/// All public functions take a `&self` and handle mutability internally.
///
/// This makes this class safe to put inside an `Arc`.
impl Server {
    /// Creates a new Server, and starts a separate thread to periodically refresh the data.
    pub fn new(config: Config) -> sync::Arc<Self> {
        let result = sync::Arc::new(Server {
            config: sync::Mutex::new(config),
            data: sync::RwLock::new(None),
            // Indicate that the refresh thread is running
            running: sync::Mutex::new(true),
        });

        // Spawn a refresh thread.
        let cloned = result.clone();
        thread::spawn(move || {
            // Refresh once immediately
            cloned.refresh();

            while *cloned.running.lock().unwrap() {
                thread::sleep(Duration::from_secs(cloned.refresh_delay()));
                cloned.refresh();
            }
        });

        result
    }

    fn refresh_delay(&self) -> u64 {
        self.config.lock().unwrap().refresh_delay.unwrap_or(30)
    }

    /// Update the configuration, and refresh everything.
    ///
    /// You should run this async (in a thread::spawn for instance).
    pub fn update_conf(&self, new: Config) {
        let mut config = self.config.lock().unwrap();
        *config = new;

        // Now let's push the script again and refresh
        prepare_hosts(&config);

        // And then just refresh
        // TODO: send a signal to the refresh thread instead?
        // TODO: write this to disk?
        self.refresh();
    }

    /// Get a read access to the latest data.
    pub fn latest_data(&self) -> sync::RwLockReadGuard<Option<Data>> {
        self.data.read().unwrap()
    }

    pub fn refresh(&self) {
        let fresh = {
            let config = self.config.lock().unwrap();
            fetch_data(&config)
        };
        let mut data = self.data.write().unwrap();
        *data = Some(fresh);
    }

    /// Stops the refresh thread.
    ///
    /// You MUST call this, or the refresh thread will
    /// keep running in the background indefinitely!
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}

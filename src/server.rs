use config::Config;
use fetch::{fetch_data, prepare_hosts};
use data::Data;

use std::sync;
use std::time;
use std::thread;
use std::time::Duration;

pub struct Server {
    config: sync::Mutex<Config>,
    data: sync::RwLock<Data>,
    running: sync::atomic::AtomicBool,
}

/// Handles concurrent access to config/data.
///
/// All public functions take a `&self` and handle mutability internally.
///
/// This makes this class safe to put inside an `Arc`.
impl Server {
    /// Creates a new Server, and starts a separate thread to periodically refresh the data.
    pub fn new(config: Config) -> sync::Arc<Self> {
        // Prepare the hosts the first time

        let result = sync::Arc::new(Server {
            config: sync::Mutex::new(config),
            data: sync::RwLock::new(Data::default()),
            // Indicate that the refresh thread is running
            running: sync::atomic::AtomicBool::new(true),
        });

        // Spawn a refresh thread.
        let cloned = result.clone();
        thread::spawn(move || {
            // Refresh once immediately
            prepare_hosts(&cloned.current_conf());
            cloned.refresh();

            // TODO: select! on a refresh channel and a timer.
            while cloned.running.load(sync::atomic::Ordering::Relaxed) {
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
        where F: FnOnce(&mut Config) -> Result<(), E>
    {

        {
            let mut config = self.current_conf();
            try!(update(&mut config));
            prepare_hosts(&config);
        }

        // And then just refresh
        // TODO: send a signal to the refresh thread instead?
        self.refresh();
        Ok(())
    }

    pub fn current_conf(&self) -> sync::MutexGuard<Config> {
        self.config.lock().unwrap()
    }

    /// Get a read access to the latest data.
    pub fn latest_data(&self) -> sync::RwLockReadGuard<Data> {
        self.data.read().unwrap()
    }

    pub fn refresh(&self) {
        // Don't lock while we're fetching.
        println!("Refreshing.");
        let start = time::Instant::now();
        let conf = self.current_conf().clone();
        let fresh = fetch_data(&conf);
        let mut data = self.data.write().unwrap();
        *data = fresh;
        println!("Refreshed ({:?})", start.elapsed());
    }

    /// Stops the refresh thread.
    ///
    /// This is called automatically on drop.
    pub fn stop(&self) {
        self.running.store(false, sync::atomic::Ordering::Relaxed);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}

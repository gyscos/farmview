use config::Config;
use fetch::{fetch_data, prepare_hosts};
use data::Data;

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
        // Prepare the hosts the first time
        prepare_hosts(&config);

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
        self.current_conf().refresh_delay.unwrap_or(30)
    }

    /// Update the configuration, and refresh everything.
    ///
    /// You should run this async (in a thread::spawn for instance).
    pub fn with_conf<E, F: FnOnce(&mut Config) -> Result<(), E>>
        (&self,
         update: F)
         -> Result<(), E> {

        let mut config = self.current_conf();
        try!(update(&mut config));

        // Now let's push the script again and refresh
        prepare_hosts(&config);
        drop(config);

        // And then just refresh
        // TODO: send a signal to the refresh thread instead?
        // TODO: write this to disk?
        self.refresh();
        Ok(())
    }

    pub fn current_conf(&self) -> sync::MutexGuard<Config> {
        self.config.lock().unwrap()
    }

    /// Get a read access to the latest data.
    pub fn latest_data(&self) -> sync::RwLockReadGuard<Option<Data>> {
        self.data.read().unwrap()
    }

    pub fn refresh(&self) {
        // Don't lock while we're fetching.
        println!("Refreshing.");
        let conf = self.current_conf().clone();
        let fresh = fetch_data(&conf);
        let mut data = self.data.write().unwrap();
        *data = Some(fresh);
        println!("Refreshed.");
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

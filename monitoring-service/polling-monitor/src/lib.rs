use std::process::Command;
use std::sync::Arc;
use std::thread::{sleep, spawn, JoinHandle, Result as ThreadResult};
use std::time::Duration;

use internal_prelude::library_prelude::*;

/// Watcher will repeatedly run a command and monitor the output of it for changes.
/// If the output changes a separate command will be triggered.
pub struct Watcher {
    cmd_to_monitor: Command,
    cmd_to_trigger: Command,
    interval:       Duration,
}

impl Watcher {
    /// Create a new watcher, will default to an interval of 1 second.
    pub fn new(cmd_to_monitor: Command, cmd_to_trigger: Command) -> Watcher {
        Watcher {
            cmd_to_monitor,
            cmd_to_trigger,
            interval: Duration::new(1, 0),
        }
    }

    /// Builder pattern for setting the interval of running the command to monitor.
    pub fn interval(&mut self, interval: Duration) -> &mut Watcher {
        self.interval = interval;
        self
    }

    /// Start the Watcher in a separate thread and return a WatcherHandle to manipulate the watcher thread.
    /// ```
    /// use std::process::Command;
    /// use watchy::Watcher;
    ///
    /// let handle = Watcher::new(Command::new("ls"), Command::new("ls")).watch();
    /// handle.stop_and_join();
    /// ```
    pub fn watch(mut self) -> WatcherHandle {
        let handle = Arc::new(Mutex::new(WatcherHandleInner::default()));
        let handle_clone = Arc::clone(&handle);

        let join_handle = spawn(move || {
            let mut previous_output = Vec::new();
            loop {
                let handle = handle.lock();
                if !handle.is_running {
                    break;
                }

                let out = self.cmd_to_monitor.output().unwrap();
                if out.stdout != previous_output {
                    self.cmd_to_trigger.spawn().unwrap();
                }
                previous_output = out.stdout;

                sleep(self.interval);
                MutexGuard::unlock_fair(handle);
            }
        });

        WatcherHandle::new(handle_clone, join_handle)
    }
}

/// WatcherHandle enables controlling the watchers thread.
pub struct WatcherHandle {
    inner:       Arc<Mutex<WatcherHandleInner>>,
    join_handle: JoinHandle<()>,
}

impl WatcherHandle {
    fn new(inner: Arc<Mutex<WatcherHandleInner>>, join_handle: JoinHandle<()>) -> Self {
        WatcherHandle { inner, join_handle }
    }

    /// Returns if the Watcher thread is semantically running.
    /// Even if this returns false in practice it may still be running it's final loop before termination.
    /// To make sure the logic loop has stopped use join() or stop_and_join().
    pub fn is_running(&self) -> bool {
        self.inner.lock().is_running
    }

    /// Signals the Watcher to stop.
    /// In practice the logic loop might still finish up it's final loop before termination.
    /// To make sure the logic loop has stopped use join() or stop_and_join().
    pub fn stop(&mut self) {
        self.inner.lock().is_running = false;
    }

    /// Join the thread running the watcher logic loop.
    pub fn join(self) -> ThreadResult<()> {
        self.join_handle.join()
    }

    /// Signal the Watcher to stop and wait for it's thread to terminate.
    pub fn stop_and_join(mut self) -> ThreadResult<()> {
        self.stop();
        self.join()
    }
}

/// Inner representation of WatcherHandle
struct WatcherHandleInner {
    is_running: bool,
}

impl Default for WatcherHandleInner {
    fn default() -> Self {
        WatcherHandleInner { is_running: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[tokio::test]
    async fn test_actual_watcher() {
        let trigger_created_file_name = "trigger_created_file";

        let mut sec_since_epoch = Command::new("date");
        sec_since_epoch.arg("+%s");

        let mut echo_done = Command::new("touch");
        echo_done.arg(trigger_created_file_name);

        let interval = Duration::from_millis(25);

        let mut watcher = Watcher::new(sec_since_epoch, echo_done);
        watcher.interval(interval);

        let watcher_handle = watcher.watch();

        sleep(Duration::from_millis(50));
        watcher_handle
            .stop_and_join()
            .expect("Join on watcher thread failed");

        let status_code = Command::new("rm")
            .arg(trigger_created_file_name)
            .output()
            .expect("Check of trigger created file failed to run")
            .status
            .code()
            .unwrap();

        assert_eq!(
            status_code, 0,
            "Watcher trigger command didn't run or didn't create a file successfully"
        );
    }
}

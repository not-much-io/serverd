use std::{
    process::Command,
    sync::Arc,
    thread::{sleep, spawn, JoinHandle, Result as ThreadResult},
    time::Duration,
};

use internal_prelude::library_prelude::*;

/// PollingMonitor will repeatedly run a command and monitor the output of it for changes.
/// If the output changes a separate command will be triggered.
pub struct PollingMonitor {
    cmd_to_monitor: Command,
    cmd_to_trigger: Command,
    interval:       Duration, // How often to run cmd_to_monitor
    delay:          Duration, // Delay before running cmd_to_trigger
}

impl PollingMonitor {
    pub fn new(cmd_to_monitor: Command, cmd_to_trigger: Command) -> PollingMonitor {
        PollingMonitor {
            cmd_to_monitor,
            cmd_to_trigger,
            interval: Duration::new(1, 0),
            delay: Duration::new(0, 0),
        }
    }

    pub fn interval(&mut self, interval: Duration) -> &mut PollingMonitor {
        self.interval = interval;
        self
    }

    pub fn delay(&mut self, delay: Duration) -> &mut PollingMonitor {
        self.delay = delay;
        self
    }

    /// Start the PollingMonitor in a separate thread and return a PollingMonitorHandle to manipulate its thread.
    /// ```
    /// use std::process::Command;
    /// use monitoring_service::PollingMonitor;
    ///
    /// let handle = PollingMonitor::new(Command::new("ls"), Command::new("ls")).watch();
    /// handle.stop_and_join();
    /// ```
    pub fn watch(mut self) -> PollingMonitorHandle {
        let handle = Arc::new(Mutex::new(PollingMonitorHandleInner::default()));
        let handle_clone = Arc::clone(&handle);

        let join_handle = spawn(move || {
            let mut previous_output = Vec::new();
            loop {
                let handle = handle.lock();
                if !handle.is_running {
                    break;
                }

                let out = self
                    .cmd_to_monitor
                    .output()
                    .expect("Failed to execute command to monitor");
                if out.stdout != previous_output {
                    self.cmd_to_trigger
                        .spawn()
                        .expect("Failed to execute command to trigger");
                }
                previous_output = out.stdout;

                sleep(self.interval);
                MutexGuard::unlock_fair(handle);
            }
        });

        PollingMonitorHandle::new(handle_clone, join_handle)
    }
}

pub struct PollingMonitorHandle {
    inner:       Arc<Mutex<PollingMonitorHandleInner>>,
    join_handle: JoinHandle<()>,
}

impl PollingMonitorHandle {
    fn new(inner: Arc<Mutex<PollingMonitorHandleInner>>, join_handle: JoinHandle<()>) -> Self {
        PollingMonitorHandle { inner, join_handle }
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
struct PollingMonitorHandleInner {
    is_running: bool,
}

impl Default for PollingMonitorHandleInner {
    fn default() -> Self {
        PollingMonitorHandleInner { is_running: true }
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

        let mut monitor = PollingMonitor::new(sec_since_epoch, echo_done);
        monitor.interval(interval);

        let monitor_handle = monitor.watch();

        sleep(Duration::from_millis(50));
        monitor_handle
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

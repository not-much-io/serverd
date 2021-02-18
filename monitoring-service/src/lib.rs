use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
    thread::{sleep, spawn, JoinHandle, Result as ThreadResult},
    time::Duration,
};

use internal_prelude::library_prelude::*;

// A significant event that should be reacted to with an action (or multiple)
#[rustfmt::skip]
pub trait Event: Eq
    + Hash
    + Send
    + 'static // TODO
    {}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct PollingSchedule {
    // This interval starts ticking down after finishing the current poll
    //
    // This is important as with very low intervals and varying duration actions
    // this could cause out of order actions to be fired
    interval: Duration,
    // TODO: Add absolute time
}

impl Default for PollingSchedule {
    fn default() -> Self {
        PollingSchedule {
            interval: Duration::new(1, 0),
        }
    }
}

impl PollingSchedule {
    pub fn interval(&mut self, interval: Duration) -> &mut Self {
        self.interval = interval;
        self
    }
}

#[rustfmt::skip]
pub trait PollingFuncInternal<E: Event>: Fn() -> Result<E>
    + Send
    + 'static // TODO
    {}

pub struct PollingFunc<E: Event>(Box<dyn PollingFuncInternal<E>>);

impl<E: Event> PollingFunc<E> {
    // fn new(f: impl PollingFuncInternal<E>) -> Self {
    //     PollingFunc(Box::new(f))
    // }
}

#[rustfmt::skip]
pub trait ActionFuncInternal: Fn() -> Result<()>
    + Send
    {}

pub struct ActionFunc(Box<dyn ActionFuncInternal>);

pub struct PollingMonitor<E: Event> {
    polling_schedule: HashMap<PollingSchedule, PollingFunc<E>>,
    event_to_actions: HashMap<E, Vec<ActionFunc>>,
}

impl<E: Event> Default for PollingMonitor<E> {
    fn default() -> Self {
        PollingMonitor {
            polling_schedule: HashMap::new(),
            event_to_actions: HashMap::new(),
        }
    }
}

impl<E: Event> PollingMonitor<E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn schedule_polling(
        &mut self,
        schedule: PollingSchedule,
        polling_func: PollingFunc<E>,
    ) -> &mut Self {
        self.polling_schedule.insert(schedule, polling_func);
        self
    }

    pub fn register_action(&mut self, event: E, action: ActionFunc) -> &mut Self {
        if let Some(actions) = self.event_to_actions.get_mut(&event) {
            actions.push(action);
        } else {
            self.event_to_actions.insert(event, vec![action]);
        }
        self
    }

    /// Start the PollingMonitor in a separate thread and return a PollingMonitorHandle to manipulate its thread.
    pub fn start(self) -> PollingMonitorHandle {
        // This is shared state between all the polling processes and the PollingMonitorHandle
        let polling_monitor_handle = Arc::new(Mutex::new(PollingMonitorHandleInner::default()));

        // NOTE: Is the mutext needed despite it being used as readonly?
        let event_to_actions = Arc::new(Mutex::new(self.event_to_actions));

        let mut polling_processes = Vec::new();
        for (schedule, polling_func) in self.polling_schedule.into_iter() {
            // Clone for this polling process
            let schedules_event_to_actions_clone = Arc::clone(&event_to_actions);
            let schedules_polling_monitor_handle_clone = Arc::clone(&polling_monitor_handle);

            let polling_process = move || -> Result<i32> {
                loop {
                    if let Ok(event) = polling_func.0() {
                        if let Some(actions) = schedules_event_to_actions_clone.lock().get(&event) {
                            for action in actions {
                                (*action.0)()?;
                            }
                        }
                    }

                    if !(schedules_polling_monitor_handle_clone).lock().is_running {
                        return Ok(0);
                    }

                    sleep(schedule.interval);
                }
            };

            polling_processes.push(polling_process);
        }

        for process in polling_processes {
            tokio::spawn(async move { process() });
        }

        PollingMonitorHandle::new(polling_monitor_handle, spawn(|| {}))
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
    // use std::{process::Command, thread::sleep};

    type MyEvent = &'static str;

    impl Event for MyEvent {}

    type MyPollingFunc = dyn Fn() -> Result<MyEvent> + Send;

    impl PollingFuncInternal<MyEvent> for MyPollingFunc {}

    // struct BasePollingFunc(dyn Fn() -> Result<String>);
    // impl PollingFuncInternal<String> for BasePollingFunc {}

    // fn poll1() -> Result<MyEvent> {
    //     Ok("")
    // }

    #[tokio::test]
    async fn test_actual_watcher() {
        // PollingMonitor::new().schedule_polling(
        //     *PollingSchedule::default().interval(Duration::new(1, 0)),
        //     PollingFunc::new(poll1),
        // );

        /*         let trigger_created_file_name = "trigger_created_file";

        let mut sec_since_epoch = Command::new("date");
        sec_since_epoch.arg("+%s");

        let mut echo_done = Command::new("touch");
        echo_done.arg(trigger_created_file_name);

        let interval = Duration::from_millis(25);

        let mut monitor = PollingMonitor::new(sec_since_epoch, echo_done);
        monitor.interval(interval);

        let monitor_handle = monitor.start();

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
        ); */
    }
}

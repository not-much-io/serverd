use std::{process::Command, str::FromStr, time::Duration};

use nursery_prelude::application_prelude::*;

use watchy::Watcher;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let key_cmd_to_monitor = "cmd-to-monitor";
    let key_cmd_to_trigger = "cmd-to-trigger";
    let key_interval = "interval";
    let matches = clap::App::new("watchy")
        .version("0.1")
        .author("kristo.koert@gmail.com")
        .about("A tool for monitoring a command for changes and triggering a separate command on change")
        .arg(clap::Arg::new(key_cmd_to_monitor)
            .long(key_cmd_to_monitor)
            .about("The command to repeatedly execute and monitor for changes")
            .value_name("COMMAND TO MONITOR")
            .required(true)
            .takes_value(true))
        .arg(clap::Arg::new(key_cmd_to_trigger)
            .long(key_cmd_to_trigger)
            .about("The command to trigger when the output of the command to monitor changes")
            .value_name("COMMAND TO TRIGGER")
            .required(true)
            .takes_value(true))
        .arg(clap::Arg::new(key_interval)
            .long(key_interval)
            .about("The interval between executing the monitoring command")
            .value_name("INTERVAL IN MS")
            .required(true)
            .takes_value(true))
        .get_matches();

    let cmd_to_monitor: Command = matches
        .value_of_t_or_exit::<CommandWrapper>(key_cmd_to_monitor)
        .into();
    let cmd_to_trigger: Command = matches
        .value_of_t_or_exit::<CommandWrapper>(key_cmd_to_trigger)
        .into();
    let interval = Duration::from_millis(matches.value_of_t_or_exit::<u64>(key_interval));

    let mut watcher = Watcher::new(cmd_to_monitor, cmd_to_trigger);
    watcher.interval(interval);

    let handle = watcher.watch();
    let _ = handle.join();
}

struct CommandWrapper(Command);

impl FromStr for CommandWrapper {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            // NOTE: Shouldn't happen with clap takes_value(true)
            return Err(anyhow!("command is empty"));
        }

        let mut split = s.split_whitespace();
        let mut cmd = Command::new(split.next().unwrap());
        for s_part in split {
            cmd.arg(s_part);
        }

        Ok(CommandWrapper(cmd))
    }
}

impl Into<Command> for CommandWrapper {
    fn into(self) -> Command {
        self.0
    }
}

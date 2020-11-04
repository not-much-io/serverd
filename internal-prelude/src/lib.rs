pub mod library_prelude {
    pub use anyhow::Result;
    pub use async_trait::async_trait;
    pub use futures;
    pub use itertools::Itertools;
    pub use lazy_static::lazy_static;
    pub use log;
    pub use parking_lot::{Mutex, MutexGuard};
    pub use regex;
    pub use thiserror;
    pub use thiserror::Error;
    pub use tokio;

    pub use crate::cli_programs::*;
}

pub mod application_prelude {
    pub use crate::library_prelude::*;

    pub use anyhow::anyhow;
    pub use anyhow::Error;
    pub use clap;
    pub use env_logger;
}

mod cli_programs {
    use std::process;

    use anyhow::Result;
    use async_trait::async_trait;

    #[async_trait]
    pub trait CLIProgram<T> {
        fn name(&self) -> &str;

        fn is_installed(&self) -> bool {
            let output = process::Command::new("which")
                .arg(&self.name())
                .output()
                .expect("failed to execute `which` command");

            !output.stdout.is_empty()
        }

        async fn call(&self) -> Result<process::Output>;

        async fn parse_output(&self, output: process::Output) -> T;
    }
}

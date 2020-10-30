pub mod library_prelude {
    mod macros {
        pub use async_trait::async_trait;
        pub use lazy_static::lazy_static;
        pub use thiserror::Error;
    }

    mod std_additions {
        pub use futures;
        pub use log;
        pub use regex;
        pub use thiserror;
        pub use tokio;
    }

    mod std_enhancements {
        pub use itertools::Itertools;
    }

    mod std_replacements {
        pub use anyhow::Result;
        pub use parking_lot::{Mutex, MutexGuard};
    }

    pub use macros::*;
    pub use std_additions::*;
    pub use std_enhancements::*;
    pub use std_replacements::*;

    pub use crate::cli_programs::*;
}

pub mod application_prelude {
    pub use crate::library_prelude::*;

    mod macros {
        pub use anyhow::anyhow;
    }

    mod std_additions {
        pub use clap;
        pub use env_logger;
    }

    mod utils {
        pub use anyhow::Error;
    }

    pub use macros::*;
    pub use std_additions::*;
    pub use utils::*;
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

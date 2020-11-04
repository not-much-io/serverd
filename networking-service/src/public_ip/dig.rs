use std::net::IpAddr;
use std::process::{Command, Output};

use internal_prelude::library_prelude::*;

use crate::public_ip::{GetPublicIP, GetPublicIPError, GetPublicIPResult};

// https://linux.die.net/man/1/dig
pub struct Dig();

impl Dig {
    fn new() -> Self {
        Dig {}
    }
}

impl Default for Dig {
    fn default() -> Self {
        Dig::new()
    }
}

impl GetPublicIP for Dig {}

#[async_trait]
impl CLIProgram<GetPublicIPResult> for Dig {
    fn name(&self) -> &str {
        "dig"
    }

    async fn call(&self) -> Result<Output> {
        Ok(Command::new(self.name())
            .arg("+short")
            .arg("myip.opendns.com")
            .arg("@resolver1.opendns.com")
            .output()?)
    }

    async fn parse_output(&self, output: Output) -> GetPublicIPResult {
        let stdout = output.stdout.clone();

        String::from_utf8(stdout[..stdout.len() - 1].to_vec())?
            .as_str()
            .parse::<IpAddr>()
            .map_err(|_| GetPublicIPError::IpParsingFailed(format!("{:?}", output)).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    const DIG_OUTPUT: &str = "95.153.16.81\n";

    #[tokio::test]
    async fn test_parse_output() {
        let output = Output {
            status: ExitStatus::from_raw(0),
            stderr: Vec::new(),
            stdout: DIG_OUTPUT.into(),
        };

        let real = Dig().parse_output(output).await.unwrap();
        let expected = DIG_OUTPUT[..DIG_OUTPUT.len() - 1]
            .parse::<IpAddr>()
            .unwrap();

        assert_eq!(real, expected);
    }

    #[tokio::test]
    async fn test_actual_call() {
        let dig = Dig();

        assert!(dig.is_installed(), "dig not installed in environment");
        if let Err(e) = dig.get_public_ip().await {
            panic!("failed to call dig with error: {}", e)
        }
    }
}

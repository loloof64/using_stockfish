use std::process::Stdio;
use std::io::Error;

use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::process::{Child, Command};

pub struct ProcessHandler {}

impl ProcessHandler {
    pub fn start_program(program_path: &String) -> Result<Child, Error> {
        let command_child = Command::new(program_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(command_child)
    }

    pub async fn read_output_line(child: &mut Child) -> Option<String> {
        match &mut child.stdout {
            Some(stdout) => {
                let mut reader = BufReader::new(stdout);
                let mut buffer = String::new();
                let result = reader.read_line(&mut buffer).await;
                match result {
                    Ok(_) => Some(buffer),
                    _ => None
                }
            },
            _ => None,
        }
    }

    pub fn send_command(child: &mut Child, command: &String) {
        let command = format!("{}\n", command);

        let _ = child.stdin.as_mut().unwrap().write(command.as_bytes());
    }

    pub async fn dispose(child: &mut Child) {
        let _ = child.kill().await;
    }
}

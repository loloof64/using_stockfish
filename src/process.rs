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

    pub async fn send_command(child: &mut Child, command: &String) {
        let command = format!("{}\n", command);

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write(command.as_bytes()).await.unwrap();
            stdin.flush().await.unwrap();
        }
        else {
            println!("child process' stdin not available");
        }


    }

    pub async fn dispose(child: &mut Child) {
        if let Ok(_) = child.kill().await {
            println!("Killed child process");
        }
    }
}

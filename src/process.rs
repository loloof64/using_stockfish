use std::io::Error;
use std::process::Stdio;

use async_std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};

pub struct ProcessHandler {}

impl ProcessHandler {
    pub async fn start_program(program_path: &String) -> Result<Child, Error> {
        let mut command_child = Command::new(program_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        command_child
            .stdin
            .as_mut()
            .unwrap()
            .write("uci\n".as_bytes())
            .await
            .unwrap();

        Ok(command_child)
    }

    pub async fn read_output_line(child: &mut Child) -> Result<String, io::Error> {
        match &mut child.stdout {
            Some(stdout) => {
                let mut result = String::new();
                let mut buf: Vec<u8> = vec![0];

                loop {
                    stdout.read(&mut buf).await?;
                    result.push(buf[0] as char);
                    if buf[0] == '\n' as u8 {
                        break;
                    }
                }

                Ok(result)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                String::from("failed to read output from process' stdout"),
            )),
        }
    }

    pub async fn send_command(child: &mut Child, command: &String) {
        let command = format!("{}\n", command);

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write(command.as_bytes()).await.unwrap();
            stdin.flush().await.unwrap();
        } else {
            println!("child process' stdin not available");
        }
    }

    pub async fn dispose(child: &mut Child) {
        if let Ok(_) = child.kill().await {
            println!("Killed child process");
        }
    }
}

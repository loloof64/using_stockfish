use std::io::{BufRead, Error, Write};
use std::process::{Child, Command, Stdio};

use tokio::io::BufReader;
use tokio::process::ChildStdout;

pub struct ProcessHandler {}

impl ProcessHandler {
    pub fn start_program(program_path: &String) -> Result<Child, Error> {
        let command_child = Command::new(program_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(command_child)
    }

    pub async fn read_output_line(child: &mut Child) -> String {
        let stdout = child.stdout.take().unwrap();
        let stdout = ChildStdout::from_std(stdout).unwrap();
        let mut result = String::new();
        let buffer = BufReader::new(stdout);
        buffer.buffer().read_line(&mut result).unwrap();
        result
    }

    pub fn send_command(child: &mut Child, command: String) {
        let command = format!("{}\n", command);

        child
            .stdin
            .as_mut()
            .unwrap()
            .write(command.as_bytes())
            .unwrap();
    }

    pub fn dispose(child: &mut Child) {
        child.kill().unwrap();
    }
}

use std::io::{BufRead, BufReader, Error, Write};
use std::process::{Child, Command, Stdio};

pub struct ProcessHandler {}

impl ProcessHandler {
    pub fn start_program(program_path: &String) -> Result<Child, Error> {
        let command_child = Command::new(program_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(command_child)
    }

    pub fn read_output(child: &mut Child) -> Vec<String> {
        let stdout = child
            .stdout
            .as_mut()
            .expect("failed to get process' stdout");
        BufReader::new(stdout)
            .lines()
            .map(|elt| match elt {
                Ok(line) => line,
                _ => String::new(),
            })
            .collect()
    }

    pub fn send_command(child: &mut Child, command: String) {
        let command = format!("{}\n", command);

        child
            .stdin
            .as_mut()
            .expect("failed to get process' stdin")
            .write(command.as_bytes())
            .expect("failed to send command to process");
    }

    pub fn dispose(child: &mut Child) {
        child.kill().expect("failed to kill child process");
    }
}

use std::cell::RefCell;
use std::io::{Error, Write, BufReader, BufRead};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

pub struct ProcessHandler {
    child: Arc<RefCell<Option<Child>>>,
}

impl ProcessHandler {
    pub fn new() -> Self {
        Self {
            child: Arc::new(RefCell::new(None)),
        }
    }

    pub fn start_program(&mut self, program_path: &String) -> Result<(), Error> {
        let command_child = Command::new(program_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        *self.child.borrow_mut() = Some(command_child);

        Ok(())
    }

    pub fn read_output(&mut self) -> Vec<String> {
        match *self.child.borrow_mut() {
            Some(ref mut child) => {
                let stdout = child.stdout.as_mut().expect("failed to get process' stdout");
                BufReader::new(stdout).lines().map(|elt| match elt {
                    Ok(line) => line,
                    _ => String::new()
                }).collect()
            },
            _ => Vec::new()
        }
    }

    pub fn send_command(&mut self, command: String) {
        let command = format!("{}\n", command);
        match *self.child.borrow_mut() {
            Some(ref mut child) => {
                child
                    .stdin
                    .as_mut()
                    .expect("failed to get process' stdin")
                    .write(command.as_bytes())
                    .expect("failed to send command to process");
            }
            _ => (),
        };
    }

    pub fn dispose(&mut self) {
        match *self.child.borrow_mut() {
            Some(ref mut child) => child.kill().expect("failed to kill child process"),
            _ => (),
        };
    }
}

use crossbeam_channel::{unbounded, Receiver};
use std::io::{Error, Read, Write};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

pub struct ProcessHandler {

}

impl ProcessHandler {
    pub fn start_program(program_path: String) -> Result<(Child, Receiver<String>), Error> {
        let mut command_child = Command::new(program_path).spawn()?;
        let mut stdout = command_child.stdout.take().unwrap();
        let (tx1, rx1) = unbounded();

        thread::spawn(move || {
            let mut buffer = [0; 200];
            let mut line_rest = String::from("");
            loop {
                stdout.read(&mut buffer).expect("failed to read data from process output");
                let mut line = match std::str::from_utf8(&buffer) {
                    Ok(content) => String::from(content),
                    _ => break,
                };
                if !line_rest.is_empty() {
                    line = format!("{}{}", line_rest, line);
                    line_rest = String::from("");
                }
                if line.is_empty() {
                    continue;
                }
                let line = String::from(line);
                let mut lines: Vec<String> =
                    line.split('\n').map(|elt| String::from(elt)).collect();
                line_rest = lines
                    .pop()
                    .expect("failed to get last line of process' stdout");
                lines.iter().for_each(|curent_line| {
                    tx1.send(curent_line.clone()).expect("failed to send data to process listener");
                });
                thread::sleep(Duration::from_millis(25));
            }
        });

        Ok((command_child, rx1))
    }

    pub fn send_command(command: String, child: &mut Child) {
        let command = format!("{}\n", command);
        let mut command_input = child.stdin.take().unwrap();
        command_input.write(command.as_bytes()).expect("failed to write command to process input");
        command_input.flush().expect("failed to flush process input");
    }
}

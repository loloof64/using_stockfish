use std::io::{Error, Write, BufReader, BufRead};
use std::process::{Child, Command};
use std::sync::mpsc::{Receiver, channel, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

pub struct ProcessHandler {

}

impl ProcessHandler {
    pub fn start_program(program_path: String) -> Result<(Child, Sender<String>, Receiver<String>), Error> {
        let command_child = Command::new(program_path).spawn()?;
        let stdout = command_child.stdout.expect("failed to get stdout from external command");
        let mut stdin = command_child.stdin.expect("failed to get stdin from external command");
        let (tx1, rx1) = channel::<String>();
        let (tx2, rx2) = channel();

        thread::spawn(move || {
            let mut f = BufReader::new(stdout);
            loop {
                match rx1.try_recv() {
                    Ok(line) => {
                        stdin.write_all(line.as_bytes()).unwrap();
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(25));
                        continue;
                    }
                    Err(e) => {
                        println!("failed to read input from program's stdin : {:?}", e);
                    }
                }
                let mut buffer = String::new();
                match f.read_line(&mut buffer) {
                    Ok(_) => {
                        tx2.send(buffer).expect("failed to send line to main code");
                    },
                    Err(e) => eprintln!("failed to send line to main code : {}", e),
                }
            }
        });

        Ok((command_child, tx1, rx2))
    }

    pub fn send_command(command: String, sender: &mut Sender<String>) {
        let command = format!("{}\n", command);
        sender.send(command).expect("failed to write command to process input");
    }
}

use std::io::{BufRead, BufReader, Write, self, ErrorKind, Error};
use std::process::{Child, Stdio, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

use std::thread;
use std::thread::sleep;
use std::time::Duration;

use signal_hook::consts::SIGTERM;


pub struct ProcessHandler {
    tx2: Sender<String>,
}

impl ProcessHandler {
   pub fn start(program_path: String) -> Result<Self, Error> {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let mut child = match ProcessHandler::start_process(program_path, tx1, rx2) {
            Ok(c) => c,
            _ => return Result::Err(Error::new(ErrorKind::Other, "Could not run program")),
        };

        thread::spawn(move || {
            let should_terminate = Arc::new(AtomicBool::new(false));
            signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate)).unwrap();
    
            while !should_terminate.load(Ordering::Relaxed) {
                match rx1.try_recv() {
                    Ok(line) => {
                        println!("Got this back: {}", line);
                    }
                    Err(TryRecvError::Empty) => {
                        sleep(Duration::from_secs(1));
                        continue;
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
    
            child.kill().unwrap();
        });
    
        let tx22 = tx2.clone();
    
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            tx22.send(String::from("uci\n")).unwrap();
            tx22.send(String::from("isready\n")).unwrap();
        });
    

        Ok(Self {
            tx2
        })
    }

    pub fn get_channel_transmitter(&self) -> Sender<String> {
        self.tx2.clone()
    }

    fn start_process(program_path: String, sender: Sender<String>, receiver: Receiver<String>) -> io::Result<Child> {
        let mut child = Command::new(program_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
    
        ProcessHandler::start_process_thread(&mut child, sender, receiver);
    
        Ok(child)
    }

    fn start_process_thread(child: &mut Child, sender: Sender<String>, receiver: Receiver<String>) {
        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        thread::spawn(move || {
            let mut f = BufReader::new(stdout);
            loop {
                match receiver.try_recv() {
                    Ok(line) => {
                        stdin.write_all(line.as_bytes()).unwrap();
                    }
                    Err(TryRecvError::Empty) => {
                        sleep(Duration::from_secs(1));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error! : {:?}", e);
                    }
                }
                let mut buf = String::new();
                match f.read_line(&mut buf) {
                    Ok(_) => {
                        sender.send(buf).unwrap();
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error! : {:?}", e);
                        break;
                    }
                }
            }
        });
    }
}
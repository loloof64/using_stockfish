mod process;
use std::{cell::RefCell, io::Error, rc::Rc};

use process::*;

mod file_explorer;
use file_explorer::*;

use dioxus::prelude::*;

fn main() {
    dioxus_desktop::launch(App);
}

fn start_program(program_path: String) -> Result<ProcessHandler, Error> {
    ProcessHandler::start(program_path)
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    let program_path = use_state(cx, || "".to_string());
    let command = use_state(cx, || "".to_string());
    let is_selecting_program = use_state(cx, || false);

    let process_handler: Rc<RefCell<Option<ProcessHandler>>> = Rc::new(RefCell::new(None));
    let process_handler_2 = process_handler.clone();

    if *is_selecting_program.current() {
        cx.render(rsx! {FileExplorer(cx)})
    } else {
        cx.render(rsx! {div {
            input {
                value: "{command}",
                oninput: move |evt| command.set(evt.value.clone())
            }
            button {
                onclick: move |_| {
                    match *process_handler_2.borrow_mut() {
                        Some(ref handler) => {
                            handler.get_channel_transmitter().send(command.to_string()).unwrap();
                        },
                        _ => {},
                    }
                },
                "Send command"
            }
        }
        div {
            input {
                value: "{program_path}",
                oninput: move |evt| program_path.set(evt.value.clone())
            }
            button {
                onclick: move |_| is_selecting_program.set(true),
                "Select program"
            }
        }
        button {
            onclick: move |_| {
                match start_program(program_path.to_string()) {
                Ok(handler) => {
                    *process_handler.borrow_mut() = Some(handler);
                }
                Err(e) => eprintln!("{}", e),
            }
        },
            "Start program"
        }})
    }
}

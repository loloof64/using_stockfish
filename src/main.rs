mod process;

use std::{cell::RefCell, process::Child, rc::Rc};

use crossbeam_channel::Receiver;
use dioxus_desktop::{Config, WindowBuilder};
use process::*;

mod file_explorer;
use file_explorer::*;

use dioxus::prelude::*;

fn main() {
    dioxus_desktop::launch_cfg(
        App,
        Config::default().with_window(WindowBuilder::new().with_title("Using Stockfish POC")),
    );
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    let program_path = use_state(cx, || "".to_string());
    let command = use_state(cx, || "".to_string());
    let is_selecting_program = use_state(cx, || false);

    let process_child = Rc::new(RefCell::new(Option::<Child>::None));
    let process_input = Rc::new(RefCell::new(Option::<Receiver<String>>::None));

    if *is_selecting_program.current() {
        cx.render(rsx! {
            FileExplorer {
                on_cancel: |_| is_selecting_program.set(false),
                on_validate: |path_string| {
                    is_selecting_program.set(false);
                    program_path.set(path_string);
                },
            }
        })
    } else {
        let process_child_clone = process_child.clone();
        let process_child_clone_2 = process_child.clone();
        cx.render(rsx! {div {
            style { include_str!("./style.css") }
            div {
                class: "fieldsLine",
                input {
                    value: "{command}",
                    oninput: move |evt| command.set(evt.value.clone())
                }
                button {
                    onclick: move |_| {
                        if let Some(ref mut child) = *process_child_clone.borrow_mut() {
                            ProcessHandler::send_command(command.to_string(), child);
                        }
                    },
                    "Send command"
                }
            }
        }
        div {
            class: "fieldsLine",
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
                match ProcessHandler::start_program(program_path.to_string()) {
                    Ok((child, receiver)) => {
                        *process_child_clone_2.borrow_mut() = Some(child);
                        *process_input.borrow_mut() = Some(receiver);
                    },
                    Err(e) => eprintln!("{}", e),
                }
            },
            "Start program"
        },
        })
    }
}

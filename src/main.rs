mod process;

use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus_desktop::{Config, WindowBuilder};

use process::*;

mod file_explorer;
use file_explorer::*;

use dioxus::prelude::*;

mod hooks;
use hooks::*;

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

    let process_handler = Rc::new(RefCell::new(ProcessHandler::new()));
    let process_handler_clone = process_handler.clone();
    let process_handler_clone_2 = process_handler.clone();
    let process_handler_clone_3 = process_handler.clone();

    use_component_lifecycle(
        cx,
        move || (),
        move || {
            process_handler.borrow_mut().dispose();
        },
    );

    use_future(cx, (), move |_| async move {
        loop {
            let lines = process_handler_clone_3.borrow_mut().read_output();
            lines.into_iter().for_each(|line| println!("{}", line));
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

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
                        process_handler_clone_2.borrow_mut().send_command(command.to_string());
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
                match process_handler_clone.borrow_mut().start_program(program_path.get()) {
                    Err(e) => eprintln!("{}", e),
                    _ => ()
                }
            },
            "Start program"
        },
        })
    }
}

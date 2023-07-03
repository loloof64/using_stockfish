mod process;

use std::{
    process::Child,
    sync::{Arc, Mutex},
    time::Duration,
};

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

    let process_handler = use_state(cx, || Arc::new(Mutex::new(Option::<Child>::None)));
    let process_handler_clone = process_handler.clone();
    let process_handler_clone_2 = process_handler.clone();
    let process_handler_clone_3 = process_handler.clone();

    use_component_lifecycle(
        cx,
        move || (),
        move || match process_handler_clone.get().try_lock() {
            Ok(ref mut locked_child) => match locked_child.as_mut() {
                Some(child) => ProcessHandler::dispose(child),
                _ => (),
            },
            _ => (),
        },
    );

    use_future(cx, (), move |_| async move {
        loop {
            match process_handler_clone_2.get().try_lock() {
                Ok(ref mut locked_child) => match locked_child.as_mut() {
                    Some(child) => {
                        let line = ProcessHandler::read_output_line(
                            child,
                        )
                        .await;
                        println!("{}", line);
                        tokio::time::sleep(Duration::from_millis(8)).await;
                    }
                    _ => (),
                },
                _ => ()
            }
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
                        match process_handler_clone_3.get().try_lock(){
                            Ok(ref mut locked_child) => match locked_child.as_mut() {
                                Some(child) => ProcessHandler::send_command(child, command.to_string()),
                                _ => ()
                            }   ,
                            _ => ()
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
                match ProcessHandler::start_program(program_path.get()) {
                    Ok(new_child) => match process_handler.get().try_lock() {
                        Ok(ref mut locked_child) => match locked_child.as_mut() {
                            Some(child) => *child = new_child,
                            _ => ()
                        },
                        _ => (),
                    },
                    Err(e) => eprintln!("{}", e),
                }
            },
            "Start program"
        },
        })
    }
}

use std::time::Duration;

use dioxus_desktop::{Config, WindowBuilder};

mod process;
use process::*;

mod file_explorer;
use file_explorer::*;

use dioxus::prelude::*;
use tokio::process::Child;

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

    let _ = use_future(
        cx,
        (program_path,),
        |(program_path,)| {
            let program_path = program_path.to_owned();
            let mut process_child = Option::<Child>::None;
            async move {
                if let Some(ref mut child) = process_child {
                    ProcessHandler::dispose(child).await;
                }
                if !program_path.is_empty() {
                    loop {
                        match process_child {
                            Some(ref mut wrapped_child) => {
                                let line =
                                    ProcessHandler::read_output_line(wrapped_child).await;
                                if let Some(line) = line {
                                    println!("{}", line);
                                }
                            }
                            _ => {
                                let command_child = ProcessHandler::start_program(&program_path);
                                match command_child {
                                    Ok(command_child) => {
                                        process_child = Some(command_child);
                                    }
                                    _ => println!("failed to run program"),
                                }
                            }
                        }
                        async_std::task::sleep(Duration::from_millis(25)).await;
                    }
                }
            }
        },
    );

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

                    },
                    "Send command"
                }
            }
        }
        div {
            class: "fieldsLine",
            input {
                value: "{program_path}",
                readonly: true,
                oninput: move |evt| program_path.set(evt.value.clone())
            }
            button {
                onclick: move |_| is_selecting_program.set(true),
                "Select program"
            }
        }
        button {
            onclick: move |_| {

            },
            "Start program"
        },
        })
    }
}

use std::{
    sync::mpsc::{channel, Sender},
    time::Duration,
};

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
    let child_input = use_state(cx, || Option::<Sender<String>>::None);

    let _ = use_future(cx, (program_path,), |(program_path,)| {
        to_owned![program_path];
        let mut process_child = Option::<Child>::None;
        let (tx, rx) = channel();
        child_input.set(Some(tx));

        async move {
            loop {
                /*
                if let Some(ref mut child) = process_child {
                    ProcessHandler::dispose(child).await;
                }
                */

                if !program_path.is_empty() {
                    match process_child {
                        Some(ref mut wrapped_child) => {
                            let line = ProcessHandler::read_output_line(wrapped_child).await;
                            if let Ok(line) = line {
                                if !line.is_empty() {
                                    print!("{}", line);
                                }
                            }
                            if let Ok(line) = rx.try_recv() {
                                ProcessHandler::send_command(wrapped_child, &line).await;
                            }
                        }
                        _ => {
                            let command_child = ProcessHandler::start_program(&program_path).await;
                            match command_child {
                                Ok(command_child) => {
                                    process_child = Some(command_child);
                                }
                                _ => println!("failed to run program"),
                            }
                        }
                    }
                }
                async_std::task::sleep(Duration::from_millis(5)).await;
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
                    onchange: move |evt| command.set(evt.value.clone())
                }
                button {
                    onclick: |_| {
                        if let Some(process_input) = child_input.get() {
                            if let Ok(_) = process_input.send(command.get().clone()) {

                            }
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
                readonly: true,
                oninput: move |evt| program_path.set(evt.value.clone())
            }
            button {
                onclick: move |_| is_selecting_program.set(true),
                "Select program"
            }
        }
        })
    }
}

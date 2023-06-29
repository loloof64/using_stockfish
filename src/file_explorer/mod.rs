use std::{
    cmp::Ordering,
    path::{self, Path, PathBuf},
};

use dioxus::prelude::*;

#[derive(Props)]
pub struct FileExplorerProps<'a> {
    on_cancel: EventHandler<'a>,
    on_validate: EventHandler<'a, String>,
}

#[allow(non_snake_case)]
pub fn FileExplorer<'a>(cx: Scope<'a, FileExplorerProps<'a>>) -> Element {
    let files = use_ref(cx, Files::new);
    let hidden_files_shown = use_ref(cx, || false);

    render!(div {
        style { include_str!("./style.css") }
        header {
            h1 { "{files.read().current()}" }
            span { }
            p {
                "Show hidden items"
            }
            input {
                "type": "checkbox",
                checked: "{hidden_files_shown.read()}",
                onchange: |evt| {
                    hidden_files_shown.set(evt.value.clone().trim().parse().unwrap());
                    files.write().reload_path_list(*hidden_files_shown.read());
                }
            }

            div { 
                onclick: move |_| files.write().go_up(*hidden_files_shown.read()),
                img {
                    class: "goUp",
                    src: "public/assets/arrow_upward.svg",
                    width: 25,
                    height: 25,
                }
            }

            div { 
                onclick: move |_| cx.props.on_cancel.call(()),
                img {
                    class: "cancel",
                    src: "public/assets/close.svg",
                    width: 25,
                    height: 25,
                }
            }

            div { 
                onclick: move |_| {
                    if files.read().selected_item_name.is_some() {
                        let item_name = String::from(files.read().selected_item_name.as_ref().unwrap());
                        let path = format!("{}{}{}", files.read().current(), path::MAIN_SEPARATOR, item_name);
                        cx.props.on_validate.call(path);
                    }
                },
                img {
                    class: "validate",
                    src: "public/assets/done.svg",
                    width: 25,
                    height: 25,
                }
            }
        }
        main {
            files.read().path_names.iter().enumerate().map(|(dir_id, path)| {
                let item_name = path.split(path::MAIN_SEPARATOR).last().unwrap();
                let path_obj = Path::new(path);
                let icon = if path_obj.is_dir() {
                    rsx!(img {
                        class: "item",
                        src: "public/assets/folder.svg",
                        width: 70,
                        height: 70,
                    })
                } else {
                    rsx!(img {
                        class: "item",
                        src: "public/assets/description.svg",
                        width: 70,
                        height: 70,
                    })
                };

                let item_class = match files.read().selected_item_name {
                    Some(ref name) => if name == item_name { "folder selected" } else { "folder"},
                    _ => "folder"
                };

                rsx! (
                    div { class: item_class, key: "{path}",
                        i { 
                            onclick: move |_| files.write().select_item(dir_id, *hidden_files_shown.read()),
                            icon,
                            p { class: "cooltip", "0 folders / 0 files" }
                        }
                        h1 { "{item_name}" }
                    }
                )
            })
            files.read().err.as_ref().map(|err| {
                rsx! (
                    div {
                        code { "{err}" }
                        button { onclick: move |_| files.write().clear_err(), "x" }
                    }
                )
            })
        }

    })
}

struct Files {
    path: PathBuf,
    path_names: Vec<String>,
    err: Option<String>,
    selected_item_name: Option<String>,
}

use directories::UserDirs;

impl Files {
    fn new() -> Self {
        let default_path = Path::new(".");
        let start_path = UserDirs::new();
        let start_path = match start_path {
            Some(ref dirs) => match dirs.document_dir() {
                Some(dir) => dir,
                _ => default_path,
            },
            _ => default_path,
        };
        let mut files = Self {
            path: start_path.to_path_buf(),
            path_names: vec![],
            selected_item_name: None,
            err: None,
        };
        files.reload_path_list(false);

        files
    }

    fn reload_path_list(&mut self, show_hidden_files: bool) {
        let cur_path = self.path.as_path();
        let paths = match std::fs::read_dir(cur_path) {
            Ok(e) => e,
            Err(err) => {
                let err = format!("An error occured: {:?}", err);
                self.err = Some(err);
                return;
            }
        };
        let collected = paths.collect::<Vec<_>>();

        // clear the current state
        self.clear_err();
        self.path_names.clear();

        for path in collected {
            self.path_names
                .push(path.unwrap().path().display().to_string());
        }

        self.path_names.sort_by(|fst, snd| {
            let fst_file_name = fst.split(path::MAIN_SEPARATOR).last().unwrap();
            let snd_file_name = snd.split(path::MAIN_SEPARATOR).last().unwrap();

            let fst_is_dir = {
                let item_name = String::from(fst_file_name);
                let item_path = self.path.join(item_name);
                if !item_path.exists() {
                    false
                } else {
                    item_path.is_dir()
                }
            };
            let snd_is_dir = {
                let item_name = String::from(snd_file_name);
                let item_path = self.path.join(item_name);
                if !item_path.exists() {
                    false
                } else {
                    item_path.is_dir()
                }
            };

            if fst_is_dir != snd_is_dir {
                if fst_is_dir {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            } else {
                fst_file_name
                    .to_lowercase()
                    .cmp(&snd_file_name.to_lowercase())
            }
        });

        if !show_hidden_files {
            self.path_names = self
                .path_names
                .iter()
                .filter(|name| {
                    let path = Path::new(name);
                    !path.file_name().unwrap().to_str().unwrap().starts_with('.')
                })
                .map(|e| e.clone())
                .collect::<Vec<_>>();
        }
    }

    fn go_up(&mut self, show_hidden_files: bool) {
        self.selected_item_name = None;
        if self.path.parent().is_some() {
            self.path = self.path.parent().unwrap().to_path_buf();
        }
        self.reload_path_list(show_hidden_files);
    }

    fn select_item(&mut self, dir_id: usize, show_hidden_files: bool) {
        let path_name = &self.path_names[dir_id];
        let path = Path::new(self.path.as_path()).join(path_name).to_path_buf();
        if path.is_file() {
            let new_selected_item_name = String::from(path.file_name().unwrap().to_str().unwrap());
            self.selected_item_name = match &self.selected_item_name {
                Some(name) => if *name == new_selected_item_name { None } else { Some(new_selected_item_name)},
                _ => Some(new_selected_item_name),
            };
        }
        else {
            self.selected_item_name = None;
            self.path = path;
            self.reload_path_list(show_hidden_files);
        }
    }

    fn current(&self) -> String {
        match self.path.as_path().to_str() {
            Some(path) => String::from(path),
            _ => String::from(path::MAIN_SEPARATOR),
        }
    }
    fn clear_err(&mut self) {
        self.err = None;
    }
}

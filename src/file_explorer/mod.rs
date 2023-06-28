use std::{
    cmp::Ordering,
    path::{self, Path, PathBuf},
};

use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn FileExplorer(cx: Scope) -> Element {
    let files = use_ref(cx, Files::new);
    let hidden_files_shown = use_ref(cx, || false);

    render!(div {
        link { href:"https://fonts.googleapis.com/icon?family=Material+Icons", rel:"stylesheet" }
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

            i { class: "material-icons", onclick: move |_| files.write().go_up(*hidden_files_shown.read()), "arrow_upward" }
        }
        main {
            files.read().path_names.iter().enumerate().map(|(dir_id, path)| {
                let item_name = path.split(path::MAIN_SEPARATOR).last().unwrap();
                let path_obj = Path::new(path);
                let icon_type = if path_obj.is_dir() {
                    "folder"
                } else {
                    "description"
                };

                rsx! (
                    div { class: "folder", key: "{path}",
                        i { class: "material-icons",
                            onclick: move |_| files.write().enter_dir(dir_id, *hidden_files_shown.read()),
                            "{icon_type}"
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
        if self.path.parent().is_some() {
            self.path = self.path.parent().unwrap().to_path_buf();
        }
        self.reload_path_list(show_hidden_files);
    }

    fn enter_dir(&mut self, dir_id: usize, show_hidden_files: bool) {
        let path_name = &self.path_names[dir_id];
        self.path = Path::new(self.path.as_path()).join(path_name).to_path_buf();
        self.reload_path_list(show_hidden_files);
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

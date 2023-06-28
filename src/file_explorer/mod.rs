use std::{path::{Path, PathBuf, self}};

use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn FileExplorer(cx: Scope) -> Element {
    let files = use_ref(cx, Files::new);

    render!(div {
        link { href:"https://fonts.googleapis.com/icon?family=Material+Icons", rel:"stylesheet" }
        style { include_str!("./style.css") }
        header {
            i { class: "material-icons icon-menu", "menu" }
            h1 { "{files.read().current()}" }
            span { }
            i { class: "material-icons", onclick: move |_| files.write().go_up(), "logout" }
        }
        main {
            files.read().path_names.iter().enumerate().map(|(dir_id, path)| {
                let path_end = path.split('/').last().unwrap_or(path.as_str());
                let icon_type = if path_end.contains('.') {
                    "description"
                } else {
                    "folder"
                };
                rsx! (
                    div { class: "folder", key: "{path}",
                        i { class: "material-icons",
                            onclick: move |_| files.write().enter_dir(dir_id),
                            "{icon_type}"
                            p { class: "cooltip", "0 folders / 0 files" }
                        }
                        h1 { "{path_end}" }
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
        let start_path = match  start_path {
            Some (ref dirs) => match dirs.document_dir() {
                Some(dir) => dir,
                _ => default_path
            },
            _ => default_path
        };
        let mut files = Self { 
            path: start_path.to_path_buf(),
            path_names: vec![],
            err: None,
        };
        files.reload_path_list();

        files
    }

    fn reload_path_list(&mut self) {
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
    }

    fn go_up(&mut self) {
        if self.path.parent().is_some() {
            self.path = self.path.parent().unwrap().to_path_buf();
        }
        self.reload_path_list();
    }

    fn enter_dir(&mut self, dir_id: usize) {
        let path_name = &self.path_names[dir_id];
        self.path = Path::new(self.path.as_path()).join(path_name).to_path_buf();
        self.reload_path_list();
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
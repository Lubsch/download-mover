use std::{
    fs::read_to_string,
    path::{Path,PathBuf},
    ffi::{OsString,OsStr},
    collections::HashMap,
    process::{Child,Command},
};

use inotify::{
    Event,
    EventMask,
};

// FILE DOWNLOAD LOG (Cancelled)
//
// File CREATE: Some("5Hr3wqFT.iso.part")
// File CREATE: Some("archlinux-2023.10.14-x86_64.iso")                 <-- size = 0
// File MOVED_FROM: Some("5Hr3wqFT.iso.part")
// File MOVED_TO: Some("archlinux-2023.qiVZ7135.10.14-x86_64.iso.part")
// File DELETE: Some("archlinux-2023.qiVZ7135.10.14-x86_64.iso.part")
// File DELETE: Some("archlinux-2023.10.14-x86_64.iso")

// FILE DOWNLOAD LOG (Complete)
//
// File CREATE: Some("SaeCHmY_.part")
// File CREATE: Some("archlinux-2023.10.14-x86_64.iso.sig")
// File MOVED_FROM: Some("SaeCHmY_.part")
// File MOVED_TO: Some("archlinux-2023.q_Yw9-ln.10.14-x86_64.iso.sig.part")
// File MOVED_FROM: Some("archlinux-2023.q_Yw9-ln.10.14-x86_64.iso.sig.part")
// File MOVED_TO: Some("archlinux-2023.10.14-x86_64.iso.sig")

// Download cancelled: DownloadStarted and empty file deleted
// Download completed: DownloadStarted and moved to empty file

#[derive(Debug)]
pub enum Progress {
    Loading(Child),
    LoadingPathed(PathBuf),
    Finished(Child),
}

#[derive(Debug)]
pub enum NewFileState {
    Waiting,
    FirstPartCreated{
        part_name: OsString
    },
    EmptyFileCreated{
        empty_name: OsString,
        part_name: OsString
    }
}

#[derive(Debug)]
pub struct State {
    pub files: HashMap<OsString, Progress>,
    pub new_file: NewFileState
}

impl State {

    pub fn new() -> Self {
        State{
            files: HashMap::new(),
            new_file: NewFileState::Waiting
        }
    }
    
    pub fn process_event(&mut self, event: &Event<&OsStr>, download_dir: &Path) {
        let file_name = event.name
            .expect("Couldn't get Event name")
            .to_os_string();
        let path = download_dir.join(&file_name);

        match (&self.new_file, event.mask) {

            (NewFileState::Waiting, EventMask::CREATE) => match path.extension() {
                Some(extension) if extension == "part" => {
                    self.new_file = NewFileState::FirstPartCreated{ part_name: file_name }
                },
                Some(extension) if extension == "download-mover" => {
                    let file_name = path.file_stem().expect("Couldn't get filename from tmp file");
                    let mv_target = read_to_string(&path).expect("Couldn't read mv_target");
                    std::fs::remove_file(&path).expect("Couldn't remove tmp file");

                    let progress = self.files.remove(file_name).expect("Couldn't get progress from HashMap");
                    match progress {
                        Progress::Loading(mut child) => {
                            child.wait().expect("Couldn't wait for path_selector.");
                            self.files.insert(file_name.into(), Progress::LoadingPathed(mv_target.into()));
                        },
                        Progress::Finished(mut child) => {
                            child.wait().expect("Couldn't wait for path_selector.");
                            mv_file(file_name, &PathBuf::from(&mv_target), download_dir);
                        },
                        Progress::LoadingPathed(path) => {
                            panic!("{file_name:?} got path twice! Old: {path:?}, New: {mv_target:?}")
                        }
                    }
                },
                _ => {}
            },

            (NewFileState::Waiting, EventMask::MOVED_TO) => match self.files.remove(&file_name) {
                Some(Progress::LoadingPathed(path)) => mv_file(&file_name, &path, download_dir),
                Some(Progress::Loading(process)) => {
                    self.files.insert(file_name, Progress::Finished(process));
                }
                Some(Progress::Finished(..)) => panic!("Download of {file_name:?} finished twice"),
                None => {}
            },

            (NewFileState::Waiting, EventMask::DELETE) => {
                self.files.remove(&file_name);
            },

            (NewFileState::FirstPartCreated { part_name }, EventMask::CREATE) => match path.metadata() {
                Ok(metadata) if metadata.len() == 0 => {
                    self.new_file = NewFileState::EmptyFileCreated{
                        empty_name: file_name,
                        part_name: part_name.clone()
                    };
                },
                _ => self.new_file = NewFileState::Waiting
            },

            (NewFileState::EmptyFileCreated { empty_name, part_name }, EventMask::MOVED_FROM) => if file_name == *part_name {
                let child = select_path_dialog(empty_name, download_dir);
                self.files.insert(empty_name.clone(), Progress::Loading(child));
            },

            (NewFileState::FirstPartCreated {..}, _) | (NewFileState::EmptyFileCreated {..}, _) => {
                self.new_file = NewFileState::Waiting;
            },

            (_, _) => {}
        }
    }

}

fn mv_file(file_name: &OsStr, path: &PathBuf, download_dir: &Path) {
    fn mv(from: PathBuf, to: &PathBuf) {
        if let Err(err) = std::fs::copy(&from, to) {
            println!("Error copying: {err}");
            return;
        }
        if let Err(err) = std::fs::remove_file(from) {
            println!("Error removing: {err}");
        }
    }

    match (path.is_dir(), path.try_exists()) {
        (true, _) => {
            mv(download_dir.join(file_name), &path.join(file_name));
        },
        (false, Ok(false)) => {
            mv(download_dir.join(file_name), path);
        }
        (false, Ok(true)) => {
            println!("{path:?} already exists.");
        },
        (false, Err(error)) => {
            println!("We don't know if {path:?} exists: {error}");
        }
    }
}

fn select_path_dialog(file_name: &OsStr, download_dir: &Path) -> Child {
    // Skip first arg and use rest as command to execute
    let mut args = std::env::args();
    args.next();
    Command::new::<String>(args.next().expect("Too few arguments!"))
        .args(args.collect::<Vec<String>>())
        .arg(file_name)
        .arg(download_dir)
        .spawn()
        .expect("Couldn't spawn child for {file_name:?}")
}

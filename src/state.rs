use std::ffi::{
    OsStr,
    OsString,
};

use nix::sys::inotify::{
    InotifyEvent,
    AddWatchFlags
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

type EmptyFileName = OsString;
type PartFile1Name = OsString;
type PartFile2Name = OsString;
type FullFileName = OsString;

#[derive(Debug)]
pub enum FileState {
    FirstPartCreated(Box<PartFile1Name>),
    EmptyFileCreated(Box<EmptyFileName>, Box<PartFile1Name>),
    Started(Box<EmptyFileName>, Box<PartFile2Name>),
    Finished(Box<FullFileName>)
}

pub struct FileInfo {
    name: Box<OsStr>,
    len: u64,
    extension: Box<OsStr>
}

pub struct State(Vec<FileState>);

impl State {

    pub fn process_event(self, event: &InotifyEvent) -> Self {

        // Ignore directory events
        if event.mask.contains(AddWatchFlags::IN_ISDIR) {
            return self;
        }

        let Some(file_name) = event.name.clone() else {
            return self;
        };

        let download_dir: std::path::PathBuf = std::env::var("XDG_DOWNLOAD_DIR")
            .expect("Please set $XDG_DOWNLOAD_DIR")
            .into();

        let path = download_dir.join(file_name.clone());
        let extension = path.extension();

        let mut result: State = State(vec!());
        for file_state in self.0 {
            result.0.push(file_state.process_event(&event, &file_name));
        }
        result
    }

}

impl FileState {

    fn process_event(self, event: &InotifyEvent, file_info: &FileInfo) -> Self {

        // println!("{0:#?}: {file_name:#?}", event.mask);

        match self {
            State::Waiting => {
                if event.mask.contains(AddWatchFlags::IN_CREATE) && file_info. == Some(OsStr::new("part")) {
                    State::FirstPartCreated(Box::new(file_name))
                } else {
                    State::Waiting
                }
            },
            State::FirstPartCreated(part_file_name) => {
                let Ok(metadata) = path.metadata();
                if event.mask.contains(AddWatchFlags::IN_CREATE) && metadata.len() == 0 {
                    State::EmptyFileCreated(Box::new(file_name), part_file_name)
                } else {
                    State::Waiting
                }
            },
            State::EmptyFileCreated(empty_file_name, part_file_name) => {
                if event.mask.contains(AddWatchFlags::IN_MOVED_FROM) && file_name == *part_file_name {
                    State::DownloadStarted(empty_file_name)
                } else {
                    State::Waiting
                }
            }
            State::DownloadStarted(_) => State::Waiting
        }

    }

}

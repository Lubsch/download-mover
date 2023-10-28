extern crate inotify;


// use std::path::PathBuf;
use std::ffi::OsStr;

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

type EmptyFileName = OsStr;
type PartFileName = OsStr;

#[derive(Debug)]
pub enum State {
    Wating,
    FirstPartCreated(Box<PartFileName>),
    EmptyFileCreated(Box<EmptyFileName>, Box<PartFileName>),
    FirstPartMoved(Box<EmptyFileName>, Box<PartFileName>),
    DownloadStarted(Box<EmptyFileName>)
}

impl State {
    
    pub fn process_event(self, event: &Event<&OsStr>) -> State {
        let name = match event.name {
            Some(n) => n.to_str().expect("Failed to convert OsString {n}"),
            _ => { return self; }
        };
        
        println!("{0:#?}: {name}", event.mask);


        if event.mask.contains(EventMask::ISDIR) {
            return self;
        }


        return self;
    }

}


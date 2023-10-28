extern crate inotify;


use std::env;

use inotify::{
    EventMask,
    WatchMask,
    Inotify,
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

fn main() -> ! {
    let mut inotify = Inotify::init()
        .expect("Failed to initialize inotify");

    let current_dir : std::path::PathBuf = env::current_dir()
        .expect("Failed to determine current directory");
    println!("Current dir: {0}", current_dir.display());

    let basename : &str = current_dir.file_name()
        .expect("Couldn't get filename")
        .to_str()
        .expect("Couldn't convert to String");
    println!("Basename: {basename}");

    inotify
        .watches()
        .add(
            current_dir,
            WatchMask::MOVE | WatchMask::CREATE | WatchMask::DELETE,
        )
        .expect("Failed to add inotify watch");

    println!("Watching current directory for activity...");

    let mut buffer = [0u8; 4096];

    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        'a: for event in events {
            let name = match event.name {
                Some(n) => n.to_str(). expect("Failed to convert OsString {n}"),
                _ => { continue 'a; }
            };
            
            if event.mask.contains(EventMask::ISDIR) {
                continue 'a;
            }

            println!("{0:#?}: {name}", event.mask);
        }
    }
}

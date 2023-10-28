extern crate inotify;

mod state;
use state::State;


use inotify::{
    WatchMask,
    Inotify,
};

fn main() -> ! {
    let mut inotify = Inotify::init()
        .expect("Failed to initialize inotify");

    let current_dir : std::path::PathBuf = std::env::current_dir()
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

    let mut state = State::Wating;

    // Buffer to read 
    let mut buffer = [0u8; 4096];

    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        for event in events {
            state = state.process_event(&event);
            println!("State: {state:?}");
        }
    }
}

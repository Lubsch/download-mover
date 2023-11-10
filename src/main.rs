use inotify::{WatchMask, Inotify};

mod state;
use state::State;

fn main() -> Result<(), std::io::Error> {
    let download_dir = std::path::PathBuf::from(
        std::env::var("XDG_DOWNLOAD_DIR").expect("Please set $XDG_DOWNLOAD_DIR")
    );

    let mut inotify = Inotify::init()?;
    inotify.watches().add(
        &download_dir,
        WatchMask::CREATE | WatchMask::DELETE | WatchMask::MOVED_FROM | WatchMask::MOVED_TO
    )?;

    println!("Watching {0:#?}...", &download_dir);

    let mut state = State::new();
    let mut buffer = [0u8; 4096];

    loop {
        for event in inotify.read_events_blocking(&mut buffer)? {
            state.process_event(&event, &download_dir);
        }
    }
}


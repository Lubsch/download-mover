extern crate inotify;

use clap::Parser;

mod state;
use state::State;

use std::path::PathBuf;
use std::ffi::OsStr;

use std::env;

use inotify::{
    WatchMask,
    Inotify,
};

#[derive(Parser, Debug)]
struct Args {
    // Script to execute
    #[arg(long)]
    script: PathBuf,

    // Terminal app to execute
    #[arg(long)]
    terminal: PathBuf,

    // Additional arguments to terminal
    #[arg(long, require_equals=true)]
    terminal_arg: Option<PathBuf>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let mut inotify = Inotify::init()
        .expect("Failed to initialize inotify");

    let download_dir: PathBuf = env::var("XDG_DOWNLOAD_DIR")
        .expect("Please set $XDG_DOWNLOAD_DIR")
        .into();

    inotify
        .watches()
        .add(
            download_dir.clone(),
            WatchMask::MOVED_FROM | WatchMask::CREATE | WatchMask::DELETE,
        )
        .expect("Failed to add inotify watch");

    println!("Watching {0:#?} for activity...", download_dir);

    // Buffer to read events
    let mut buffer = [0u8; 4096];

    let mut state = State::Waiting;

    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        for event in events {
            state = state.process_event(&event).unwrap_or_else(|| {
                println!("Failed to process event: {event:#?}");
                State::Waiting
            });
            // println!("State: {state:?}");

            if let State::DownloadStarted(file_name) = state {
                select_path_dialog(&file_name, &args.script, &args.terminal, &args.terminal_arg);
                state = State::Waiting;
            }
        }
    }
}

fn select_path_dialog(file_name: &OsStr, script: &PathBuf, terminal: &PathBuf, terminal_arg: &Option<PathBuf>) {
    let mut command = std::process::Command::new(terminal);
    if let Some(arg) = terminal_arg {
        command.arg(arg);
    }
    let output = command
        .arg(script)
        .arg(file_name)
        .output()
        .expect("Failed to get script output");

    println!("{output:#?}");
}

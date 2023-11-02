use clap::Parser;

use std::path::PathBuf;
use std::ffi::OsStr;

use nix::sys::stat::Mode;
use nix::sys::inotify::{
    Inotify,
    InitFlags,
    AddWatchFlags
};

mod state;
use state::State;


#[derive(clap::Parser, Debug)]
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

    let instance = Inotify::init(InitFlags::empty())?;
    
    let download_dir: PathBuf = std::env::var("XDG_DOWNLOAD_DIR")
        .expect("Please set $XDG_DOWNLOAD_DIR")
        .into();

    instance.add_watch(
       &download_dir,
       AddWatchFlags::IN_CREATE | AddWatchFlags::IN_DELETE | AddWatchFlags::IN_MOVED_FROM
    )?;

    println!("Watching {0:#?} for activity...", download_dir);

    let mut state = State::Waiting;

    loop {
        let events = instance.read_events()?;

        for event in events {
            state = state.process_event(&event);
            // println!("State: {state:?}");

            if let State::DownloadStarted(file_name) = state {
                let path = select_path_dialog(&file_name, &args.script, &args.terminal, &args.terminal_arg)?;
                println!("{path:#?}");
                state = State::Waiting;
            }
        }
    }
}

fn select_path_dialog(
    file_name: &OsStr,
    script: &PathBuf,
    terminal: &PathBuf,
    terminal_arg: &Option<PathBuf>
) -> Result<PathBuf, std::io::Error> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_path = tmp_dir.path().join("path");
    nix::unistd::mkfifo(&tmp_path, Mode::S_IRWXU)?;
    println!("{tmp_path:?}");

    let mut command = std::process::Command::new(terminal);
    if let Some(arg) = terminal_arg {
        command.arg(arg);
    }
    command
        .arg(&script)
        .arg(&file_name)
        .arg(&tmp_path)
        .output()?;

    Ok(PathBuf::from(std::fs::read_to_string(tmp_path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_path_dialog() {
        let output = select_path_dialog(
            &OsStr::new("test.txt"),
            &PathBuf::from("../script"),
            &PathBuf::from("footclient"),
            &Some(PathBuf::from("--app-id=float"))
        );
        println!("{output:?}");
    }

}

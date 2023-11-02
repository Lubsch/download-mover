use clap::Parser;

use std::{
    io::{
        stdin,
        BufRead,
    },
    ffi::{
        OsString,
        OsStr
    },
    collections::HashMap,
    path::PathBuf,
};

use inotify::{
    EventMask,
    WatchMask,
    Inotify
};

mod state;
use state::{
    State,
    Progress
};


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
    terminal_arg: Option<OsString>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let mut inotify = Inotify::init()?;
    
    let download_dir: PathBuf = std::env::var("XDG_DOWNLOAD_DIR")
        .expect("Please set $XDG_DOWNLOAD_DIR")
        .into();

    inotify.watches().add(
       &download_dir,
       WatchMask::CREATE | WatchMask::DELETE | WatchMask::MOVED_FROM
    )?;

    println!("Watching {0:#?} for activity...", download_dir);

    let mut state = State::Waiting;
    let mut files: HashMap<OsString, Progress> = HashMap::new();

    let mut buffer = [0u8; 4096];
    loop {
        let events = inotify.read_events_blocking(&mut buffer)?;

        'process_events: for event in events {
            let Some(file_name) = event.name else {
                continue 'process_events;
            };

            match state {

                State::Waiting => {
                    if event.mask.contains(EventMask::CREATE) {
                        if let Some(extension) = download_dir.join(&file_name).extension() {
                            if extension == "part" {
                                state = State::FirstPartCreated{ part_name: file_name.into() };
                            }
                        }
                        continue 'process_events;
                    }
                    if event.mask.contains(EventMask::MOVED_TO) {
                        if let Some(progress) = files.remove(file_name.into()) {
                            match progress {
                                Progress::LoadingPathed(path) => {
                                    mv_file(&file_name, &path);
                                },
                                Progress::Loading(process) => {
                                    files.insert(file_name.into(), Progress::Finished(process));
                                },
                                Progress::Finished(..) => {
                                    panic!("Download of {file_name:?} finished twice");
                                }
                            }
                        }
                    }
                    if event.mask.contains(EventMask::DELETE) {
                        files.remove(file_name.into());
                    }
                }

                State::FirstPartCreated { part_name } => {
                    if event.mask.contains(EventMask::CREATE) {
                        if let Ok(metadata) = download_dir.join(&file_name).metadata() {
                            if metadata.len() == 0 {
                                state = State::EmptyFileCreated{
                                    empty_name: file_name.into(),
                                    part_name
                                };
                                continue 'process_events;
                            }
                        }
                    }
                    state = State::Waiting;
                }

                State::EmptyFileCreated { ref empty_name, ref part_name } => {
                    if event.mask.contains(EventMask::MOVED_FROM) {
                        if file_name == *part_name {
                            files.insert(empty_name.clone(), Progress::Loading(
                                select_path_dialog(&file_name, &args)?
                            ));
                            println!("Todo get path for {empty_name:?} from user");
                            state = State::Waiting;
                        }
                    }
                }

            }
        }

        // Read selected paths from stdin
    }
}

fn mv_file(file_name: &OsStr, path: &PathBuf) {
    println!("Would move {file_name:?} to {path:?}");
}

fn select_path_dialog(
    file_name: &OsStr,
    args: &Args
) -> Result<std::process::Child, std::io::Error> {

    let mut command = std::process::Command::new(&args.terminal);
    if let Some(arg) = &args.terminal_arg {
        command.arg(arg);
    }
   command
        .arg(&args.script)
        .arg(&file_name)
        .arg(std::process::id().to_string())
        .spawn()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_path_dialog() {
        let mut child = select_path_dialog(
            &OsString::from("test.txt"),
            &Args{
                script: PathBuf::from("../script"),
                terminal: PathBuf::from("footclient"),
                terminal_arg: Some(OsString::from("--app-id=float"))
            }
        ).unwrap();
        'a: loop {
            for line in stdin().lock().lines() {
                println!("Read {0}", line.unwrap());
                let _ = child.wait(); // Must "drop" it manually
                break 'a;
            }
        }
    }

}

mod rw_file;

use chrono::Utc;
use clap::Parser;
use log::{error, info};
use std::fs::{self, metadata};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Task manager
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Task list file path or directory
    #[arg(short, long)]
    backlog: String,

    /// Run N jobs in parallel
    #[arg(short, long, default_value_t = 1)]
    jobs: usize,

    /// Demon mode
    #[arg(short, long, default_value_t = false)]
    demon: bool,
}

fn exec(task: &str) {
    let mut params: Vec<&str> = task.split_ascii_whitespace().collect();
    let executable = params[0].to_string();
    let start_time = Utc::now();
    params.remove(0);

    match Command::new(executable)
        .args(params)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            info!(
                "{} -> {}",
                start_time.format("%Y-%m-%d %H:%M:%S%.3f%z"),
                task
            );
            println!("{stderr}");
            println!("{stdout}");
        }
        Err(e) => {
            error!("Failed to execute command: {task}. Error: {e}");
        }
    }
}

fn thread_visitor(thread_pool: &mut Vec<thread::JoinHandle<()>>) {
    let mut index = 0;
    while index < thread_pool.len() {
        if thread_pool[index].is_finished() {
            thread_pool.remove(index);
        } else {
            index += 1;
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let args = Args::parse();

    let mut has_tasks: bool = true;
    let mut is_locked: bool;
    let mut thread_pool: Vec<thread::JoinHandle<()>> = Vec::new();

    let backlog_path = PathBuf::from(&args.backlog);
    let is_dir = backlog_path.is_dir();
    let mut backlog_file: Option<PathBuf> = None;
    let mut history_file: PathBuf;

    if !backlog_path.exists() {
        error!("File or directory path \"{}\" does not exist", args.backlog);
        return Ok(());
    }

    while args.demon || has_tasks || !thread_pool.is_empty() {
        if is_dir {
            history_file = backlog_path.join("qrun_history.log");
            let mut found_file = false;
            for entry in fs::read_dir(&backlog_path)? {
                let path = entry?.path();
                let ext = path.extension().and_then(|s| s.to_str());
                if ext == Some("bl") {
                    let start_time = Utc::now();
                    info!(
                        "{} -> backlog: {}",
                        start_time.format("%Y-%m-%d %H:%M:%S%.3f%z"),
                        path.display()
                    );
                    backlog_file = Some(path);
                    found_file = true;
                    break;
                }
            }

            if !found_file {
                thread_visitor(&mut thread_pool);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        } else {
            history_file = backlog_path.parent().unwrap().join("qrun_history.log");
            backlog_file = Some(backlog_path.clone());
        }

        if let Some(file) = &backlog_file {
            if file.exists() {
                // Read the backlog
                let mut tasks = rw_file::read(&file.to_string_lossy().to_string());

                // Verify that the backlog file is not being edited
                let parent = file.parent().unwrap();
                let swapfile_filename =
                    format!(".{}.swp", file.file_name().unwrap().to_str().unwrap());

                match metadata(parent.join(&swapfile_filename)) {
                    Ok(_) => is_locked = true,
                    Err(_) => is_locked = false,
                }

                if tasks.is_empty() {
                    // No more tasks need to be executed in the backlog
                    fs::remove_file(file)?;
                    has_tasks = false;
                    backlog_file.take();
                } else if !is_locked && thread_pool.len() < args.jobs {
                    while thread_pool.len() < args.jobs && !tasks.is_empty() {
                        let task = tasks[0].to_string();
                        tasks.remove(0);
                        println!("Executing: {}", &task);
                        rw_file::append(history_file.to_string_lossy().as_ref(), &tasks)?;
                        thread_pool.push(thread::spawn(move || exec(&task)));
                    }
                    rw_file::write(&file.to_string_lossy().to_string(), &tasks)?;
                }
            }
        }

        thread_visitor(&mut thread_pool);

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

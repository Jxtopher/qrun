mod rw_file;
mod thread_pool;

use chrono::Utc;
use clap::Parser;
use log::{error, info};
use std::fs::{self, metadata};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::thread_pool::ThreadPool;

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

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let args = Args::parse();

    let mut has_tasks: bool = true;
    let mut is_locked: bool;
    let mut thread_pool: ThreadPool = ThreadPool::new(args.jobs);

    let backlog_path = PathBuf::from(&args.backlog);
    let is_dir = backlog_path.is_dir();
    let mut backlog_file: Option<PathBuf> = None;
    let mut history_file: PathBuf;

    if !backlog_path.exists() {
        error!("File or directory path \"{}\" does not exist", args.backlog);
        return Ok(());
    }

    while args.demon || has_tasks || thread_pool.is_ongoing() {
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
                // thread_visitor(&mut thread_pool);
                thread_pool.update();
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
                } else if !is_locked && thread_pool.has_one_available() {
                    while (thread_pool.has_one_available()) && (!tasks.is_empty()) {
                        let thread_id = thread_pool.get_one_available();
                        if let Some(thread_id) = thread_id {
                            let task = tasks[0].to_string();
                            tasks.remove(0);
                            println!("{} - {}", thread_id, &task);
                            rw_file::append(history_file.to_string_lossy().as_ref(), &tasks)?;
                            thread_pool.exec_task(thread_id, task);
                        }
                    }
                    rw_file::write(&file.to_string_lossy().to_string(), &tasks)?;
                }
            }
        }

        thread_pool.update();

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

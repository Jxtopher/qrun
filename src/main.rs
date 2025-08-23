mod rw_file;
mod thread_pool;

use chrono::Utc;
use clap::Parser;
use ctrlc;
use log::{error, info};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
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

    /// File containing the logs
    #[arg(short, long, default_value_t = String::from("log.txt"))]
    logpath: String,
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let args = Args::parse();

    let running = Arc::new(AtomicBool::new(true));
    {
        let running = Arc::clone(&running);
        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
            println!("\nReceived interruption (Ctrl+C), shutting down gracefully...");
        })
        .expect("Error setting up Ctrl+C handler");
    }

    let mut has_tasks: bool = true;
    let mut thread_pool: ThreadPool = ThreadPool::new(args.jobs);

    let backlog_path = PathBuf::from(&args.backlog);
    let is_dir = backlog_path.is_dir();
    let mut backlog_file: Option<PathBuf> = None;
    let mut history_file: PathBuf;

    let log_file = File::options()
        .create(true)
        .append(true)
        .open(args.logpath)?;

    if !backlog_path.exists() {
        error!("File or directory path \"{}\" does not exist", args.backlog);
        return Ok(());
    }

    if !args.demon {
        print!("{thread_pool}");
    }

    while (args.demon || has_tasks) && running.load(Ordering::SeqCst) {
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
                let mut tasks = rw_file::read(&file);

                while !tasks.is_empty() && running.load(Ordering::SeqCst) {
                    while thread_pool.has_one_available() && !tasks.is_empty() {
                        let thread_id = thread_pool.get_one_available();
                        if let Some(thread_id) = thread_id {
                            let task = tasks[0].to_string();
                            tasks.remove(0);
                            rw_file::append(&history_file, &task)?;
                            thread_pool.exec_task(thread_id, task, log_file.try_clone()?);
                        }
                    }

                    if !args.demon {
                        print!("{thread_pool}");
                    }

                    thread_pool.update();

                    thread::sleep(Duration::from_secs(1));
                }

                while thread_pool.is_ongoing() && running.load(Ordering::SeqCst) {
                    if !args.demon {
                        print!("{thread_pool}");
                    }

                    thread_pool.update();

                    thread::sleep(Duration::from_secs(1));
                }

                if !tasks.is_empty() {
                    rw_file::write(&file, &tasks)?;
                } else {
                    fs::remove_file(file)?;
                }
                has_tasks = false;
            }
        }
    }

    Ok(())
}

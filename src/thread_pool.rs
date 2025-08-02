use chrono::Utc;
use log::{error, info};
use std::process::Command;
use std::{io, thread};

fn exec(task: &str) {
    let mut params: Vec<&str> = task.split_ascii_whitespace().collect();
    let executable = params[0].to_string();
    let start_time = Utc::now();
    params.remove(0);

    match Command::new(executable)
        .args(params)
        .stdout(io::stdout())
        .stderr(io::stderr())
        .status()
    {
        Ok(_) => {
            info!(
                "{} -> {}",
                start_time.format("%Y-%m-%d %H:%M:%S%.3f%z"),
                task
            );
        }
        Err(e) => {
            error!("Failed to execute command: {task}. Error: {e}");
        }
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    tasks: Vec<String>,
    thread_count: usize,
    threads: Vec<Option<thread::JoinHandle<()>>>,
    start_time: Vec<chrono::DateTime<Utc>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut tasks = Vec::with_capacity(size);
        let mut threads = Vec::with_capacity(size);
        let mut start_time = Vec::with_capacity(size);
        for _ in 0..size {
            threads.push(None);
            tasks.push("".to_string());
            start_time.push(Utc::now());
        }
        ThreadPool {
            tasks,
            thread_count: 0,
            threads,
            start_time,
        }
    }

    pub fn is_ongoing(&self) -> bool {
        for thread in &self.threads {
            if thread.is_some() {
                return true;
            }
        }
        false
    }

    pub fn get_one_available(&self) -> Option<usize> {
        let mut index = 0;
        while index < self.threads.len() {
            if self.threads[index].is_none() {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub fn exec_task(&mut self, thread_id: usize, task: String) {
        self.tasks[thread_id] = task.clone();
        let handle = thread::spawn(move || exec(&task));
        self.threads[thread_id] = Some(handle);
        self.thread_count += 1;
        self.start_time[thread_id] = chrono::Utc::now();
    }

    pub fn update(&mut self) {
        for thread in &mut self.threads {
            if let Some(handle) = thread {
                if handle.is_finished() {
                    *thread = None;
                    self.thread_count -= 1;
                }
            }
        }
    }

    pub fn has_one_available(&self) -> bool {
        self.thread_count < self.threads.len()
    }
}

impl std::fmt::Display for ThreadPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.threads.len() {
            let task_string = match self.threads[i] {
                Some(_) => "Active".to_string(),
                None => "Idle".to_string(),
            };

            writeln!(f, "Thread {}: {} {}", i + 1, task_string, self.tasks[i])?;
        }
        Ok(())
    }
}

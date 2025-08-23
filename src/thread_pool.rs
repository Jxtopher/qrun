use chrono::Utc;
use log::{error, info};
use std::fs::File;
use std::process::Command;
use std::thread;

fn exec(task: &str, log: File) -> i32 {
    let mut params: Vec<&str> = task.split_ascii_whitespace().collect();
    let executable = params[0].to_string();
    let start_time = Utc::now();
    params.remove(0);

    match Command::new(executable)
        .args(params)
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .status()
    {
        Ok(status) => {
            info!(
                "{} -> {}",
                start_time.format("%Y-%m-%d %H:%M:%S%.3f%z"),
                task
            );
            status.code().unwrap_or(-1)
        }
        Err(e) => {
            error!("Failed to execute command: {task}. Error: {e}");
            -1
        }
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    tasks: Vec<String>,
    thread_count: usize,
    threads: Vec<Option<thread::JoinHandle<i32>>>,
    start_time: Vec<chrono::DateTime<Utc>>,
    count_tasks_processed: Vec<u64>,
    new_tasks: Vec<bool>,
    status_code: Vec<(u64, u64)>, // (count_success, count_failure)
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut tasks = Vec::with_capacity(size);
        let mut threads = Vec::with_capacity(size);
        let mut start_time = Vec::with_capacity(size);
        let mut count_tasks_processed = Vec::with_capacity(size);
        let mut new_tasks = Vec::with_capacity(size);
        let mut status_code = Vec::with_capacity(size);
        for _ in 0..size {
            tasks.push("".to_string());
            threads.push(None);
            start_time.push(Utc::now());
            count_tasks_processed.push(0);
            new_tasks.push(false);
            status_code.push((0, 0));
        }
        ThreadPool {
            tasks,
            thread_count: 0,
            threads,
            start_time,
            count_tasks_processed,
            new_tasks,
            status_code,
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

    pub fn exec_task(&mut self, thread_id: usize, task: String, log: File) {
        self.tasks[thread_id] = task.clone();
        let handle = thread::spawn(move || exec(&task, log));
        self.threads[thread_id] = Some(handle);
        self.thread_count += 1;
        self.start_time[thread_id] = chrono::Utc::now();
        self.count_tasks_processed[thread_id] += 1;
        self.new_tasks[thread_id] = true;
    }

    pub fn update(&mut self) {
        for i in 0..self.threads.len() {
            if let Some(handle) = self.threads[i].as_ref() {
                if handle.is_finished() {
                    // Prenez le handle et remplacez-le par None
                    let handle = self.threads[i].take().unwrap();
                    match handle.join() {
                        Ok(status) => {
                            if status == 0 {
                                self.status_code[i].0 += 1;
                            } else {
                                self.status_code[i].1 += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("Thread error: {:?}", e);
                        }
                    }
                    self.threads[i] = None;
                    self.thread_count -= 1;
                }
            }
        }
        for i in 0..self.new_tasks.len() {
            self.new_tasks[i] = false;
        }
    }

    pub fn has_one_available(&self) -> bool {
        self.thread_count < self.threads.len()
    }
}

impl std::fmt::Display for ThreadPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[THREAD STATUS SUCCESS FAILURE] CMD\n")?;
        for i in 0..self.threads.len() {
            let task_string = match self.threads[i] {
                Some(_) => "Active".to_string(),
                None => " Idle ".to_string(),
            };

            let start_color: &'static str = match self.new_tasks[i] {
                true => "\x1b[34m",
                false => "",
            };
            let reset_color: &'static str = match self.new_tasks[i] {
                true => "\x1b[0m",
                false => "",
            };

            write!(
                f,
                "{}[{: >6} {} {: >7}{: >8}] {}{}\n",
                start_color,
                format!("T{}", i),
                task_string,
                self.status_code[i].0,
                self.status_code[i].1,
                self.tasks[i].chars().take(48).collect::<String>(),
                reset_color
            )?;
        }
        write!(
            f,
            "[{: >6} {} {: >7}{: >8}]\n",
            self.new_tasks.iter().filter(|&&x| x).count(),
            "      ",
            self.status_code
                .iter()
                .map(|(success, _)| success)
                .sum::<u64>(),
            self.status_code
                .iter()
                .map(|(_, failure)| failure)
                .sum::<u64>(),
        )?;
        print!("\x1B[{}A", self.threads.len() + 2);

        Ok(())
    }
}

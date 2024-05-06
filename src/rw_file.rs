use log::error;
use std::fs::OpenOptions;
use std::fs::{read_to_string, File};
use std::io::{self, Write};

pub fn read(filename: &String) -> Vec<String> {
    let mut result = Vec::new();
    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string())
    }
    result
}

pub fn write(filename: &String, tasks: &[String]) {
    if tasks.is_empty() {
        // Write an empty file
        let mut data_file = File::create(filename).expect("creation failed");
        data_file.write_all(b"").expect("write failed");
    } else {
        let mut data_file = File::create(filename).expect("creation failed");
        for i in 0..=(tasks.len() - 1) {
            let mut line = tasks[i].to_string();
            if i != tasks.len() - 1 {
                line.push('\n');
            }
            match data_file.write(line.as_bytes()) {
                Ok(_) => (),
                Err(..) => {
                    error!("Write failed");
                }
            }
        }
    }
}

pub fn append(file_path: &str, content: &[String]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;

    for line in content {
        writeln!(file, "{line}")?;
    }

    Ok(())
}

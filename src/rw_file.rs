use std::fs::OpenOptions;
use std::fs::{read_to_string, File};
use std::io::{self, Write};
use std::path::PathBuf;

pub fn read(filename: &PathBuf) -> Vec<String> {
    read_to_string(filename)
        .unwrap()
        .lines()
        .map(|line| line.to_string())
        .collect()
}

pub fn write(filename: &PathBuf, tasks: &[String]) -> Result<(), io::Error> {
    let mut data_file = File::create(filename)?;

    if tasks.is_empty() {
        data_file.write_all(b"")?; // Write an empty file if no tasks are provided
    } else {
        for task in tasks {
            writeln!(data_file, "{task}")?; // Use writeln! for automatic newline insertion
        }
    }

    Ok(())
}

pub fn append(filepath: &PathBuf, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filepath)?;

    writeln!(file, "{content}")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_empty() {
        let tmpdir = TempDir::new().unwrap();
        let filename = tmpdir.path().join("test_tasks.txt");
        let tasks: Vec<String> = vec![];
        write(&filename, &tasks).unwrap();
        assert!(fs::read_to_string(&filename).unwrap().is_empty());
        fs::remove_file(&filename).unwrap();
    }

    #[test]
    fn test_write_with_tasks() {
        let tmpdir = TempDir::new().unwrap();
        let filename = tmpdir.path().join("test_tasks.txt");
        let tasks = vec!["Task 1".to_string(), "Task 2".to_string()];
        write(&filename, &tasks).unwrap();

        let content = fs::read_to_string(&filename).unwrap();
        assert_eq!(content, "Task 1\nTask 2\n");
        fs::remove_file(&filename).unwrap();
    }
}

use std::{env::current_dir, path::PathBuf};

/// Returns path to `data_in` directory, assuming the current directory is somewhere inside the project.
/// ```
/// assert!(common::utils::get_data_in_dir().to_str().unwrap().ends_with("data-in"));
/// ```
pub fn get_data_in_dir() -> PathBuf {
    get_project_dir().join("data-in")
}

/// Returns path to `data_out` directory, assuming the current directory is somewhere inside the project.
/// ```
/// assert!(common::utils::get_data_out_dir().to_str().unwrap().ends_with("data-out"));
/// ```
pub fn get_data_out_dir() -> PathBuf {
    get_project_dir().join("data-out")
}

fn get_project_dir() -> PathBuf {
    let current_dir = current_dir().unwrap();

    let mut dir = current_dir.as_path();
    loop {
        if dir.file_name().unwrap() == "somewhere-like" {
            break dir.to_path_buf();
        }
        dir = dir.parent().expect(&format!("Project root dir not found in: {:?}", current_dir));
    }
}

pub fn eprintln_memory_usage() {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let mem_bytes = sys.process(sysinfo::get_current_pid().unwrap()).unwrap().memory();
    eprintln!("Using {} MB of RAM", (mem_bytes as f64) / 1024.0 / 1024.0);
}

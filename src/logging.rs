use dirs::data_local_dir;
use env_logger::{self, Builder, Env, Target};
use std::env;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

pub fn init() {
    // only log for debug builds
    if cfg!(debug_assertions) {
        let log_path = get_log_path();

        // Ensure the parent directory exists
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let file = File::create(log_path).unwrap();
        let writer = BufWriter::new(file);

        let mut builder = Builder::from_env(Env::default().default_filter_or("error,dutar=debug"));
        builder.target(Target::Pipe(Box::new(writer)));
        builder.init();
    }
}

fn get_log_path() -> PathBuf {
    // Check for LOG_PATH environment variable first
    if let Ok(log_path) = env::var("LOG_PATH") {
        return PathBuf::from(shellexpand::tilde(&log_path).to_string());
    }

    // Default to data directory
    // Linux:   ~/.local/share/dutar/dutar.log
    // macOS:   ~/Library/Application Support/dutar/dutar.log
    // Windows: C:\Users\<user>\AppData\Roaming\dutar\dutar.log
    let mut log_path = data_local_dir().expect("Access to data directory");
    log_path.push("dutar");
    log_path.push("dutar.log");
    log_path
}

use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use chrono::Datelike;
use once_cell::sync::Lazy;

static LOG_FILE: Lazy<Mutex<File>> = Lazy::new(|| {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        std::fs::create_dir(log_dir).unwrap();
    }
    let now = chrono::Utc::now().with_timezone(&chrono_tz::Japan);
    let filename = format!("{}-{:02}-{:02}.log", now.year(), now.month(), now.day());
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_dir.join(filename))
        .unwrap();
    Mutex::new(file)
});

pub struct LogWriter;

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut file = LOG_FILE.lock().unwrap();
        writeln!(file, "{}", String::from_utf8(buf.to_vec()).unwrap())?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

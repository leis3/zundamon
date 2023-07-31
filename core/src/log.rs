use std::io::Write;
use std::fs::File;
use std::path::Path;
use chrono::Datelike;

fn open_log_file() -> std::io::Result<File> {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        std::fs::create_dir(log_dir).unwrap();
    }
    let now = chrono::Utc::now().with_timezone(&chrono_tz::Japan);
    let filename = format!("{}-{:02}-{:02}.log", now.year(), now.month(), now.day());
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_dir.join(filename))
}

pub struct LogWriter;

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut file = open_log_file()?;
        writeln!(file, "{}", String::from_utf8(buf.to_vec()).unwrap())?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

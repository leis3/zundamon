use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use anyhow::Result;
use serenity::{
    http::Http,
    model::webhook::Webhook
};

pub static LOG_WEBHOOK: OnceCell<String> = OnceCell::new();

pub static BUF: OnceCell<Mutex<Vec<u8>>> = OnceCell::new();

pub struct LogWriter;

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lock = BUF.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
        lock.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut lock = BUF.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
        lock.flush()
    }
}

pub async fn send_log() -> Result<()> {
    let content = {
        let Some(m) = BUF.get() else {
            anyhow::bail!("Nothing is logged");
        };
        let mut lock = m.lock().unwrap();
        let data = lock.drain(..).collect::<Vec<_>>();
        let striped = strip_ansi_escapes::strip(data)?;
        String::from_utf8(striped)?
    };
    if let Some(url) = LOG_WEBHOOK.get() {
        let http = Http::new("");
        let webhook = Webhook::from_url(&http, url).await?;
        webhook.execute(&http, false, |w| w.content(content)).await?;
    }
    Ok(())
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*);
        let _ = $crate::log::send_log().await;
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
        let _ = $crate::log::send_log().await;
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
        let _ = $crate::log::send_log().await;
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
        let _ = $crate::log::send_log().await;
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
        let _ = $crate::log::send_log().await;
    };
}

use std::time::SystemTime;
use log::{Log, Record, LevelFilter, Level, Metadata, set_boxed_logger, set_max_level};
use tokio::sync::mpsc;


/// The struct for a log message.
#[derive(Debug)]
pub struct LogMsg {
    level: Level,
    time: SystemTime,
    msg: String,
}

/// The Logger struct with a unbounded channel.
#[derive(Clone)]
pub struct Logger {
    level: LevelFilter,
    tx: mpsc::UnboundedSender<LogMsg>,
}

impl Logger {
    pub fn new(level: LevelFilter) -> (Box<Logger>, mpsc::UnboundedReceiver<LogMsg>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let logger = Box::new(Logger{level, tx});
        set_max_level(level);
        set_boxed_logger(logger.clone()).unwrap();
        (logger, rx)
    }
    pub async fn run(self, mut rx: mpsc::UnboundedReceiver<LogMsg>) -> Result<(), std::io::Error> {
        while let Some(m) = rx.recv().await {
            let LogMsg{level, time, msg} = m;
            println!("{:?} - {} - {}", time, level, msg);
        }
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = SystemTime::now();
            let _ = self.tx.send(LogMsg{
                level: record.level(),
                time: now,
                msg: record.args().to_string()
            });
        }
    }
    fn flush(&self) {}
}

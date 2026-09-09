use barnacle_lib::fs::state_dir;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_rolling_file::RollingFileAppender;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

const MAX_LOG_FILE_COUNT: usize = 8;
const MAX_LOG_FILE_SIZE: u64 = 10 * 1024 * 1024;

pub fn setup() -> WorkerGuard {
    let log_file_basename = state_dir()
        .join("logs")
        .join("barnacle.log")
        .to_str()
        .unwrap()
        .to_string();

    let file_appender = RollingFileAppender::builder()
        .filename(log_file_basename)
        .max_filecount(MAX_LOG_FILE_COUNT)
        .condition_max_file_size(MAX_LOG_FILE_SIZE)
        .build()
        .unwrap();

    let (file_writer, guard) = file_appender.get_non_blocking_appender();

    let filter = EnvFilter::new("barnacle_gui=info,barnacle_lib=info,barnacle_cli=info,warn");

    FmtSubscriber::builder()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_env_filter(filter)
        .init();

    guard
}

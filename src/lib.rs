pub mod duplicates;
pub mod duplicates_rayon;
pub mod error;
pub mod image;
pub mod indexer;
pub mod similarity;

use std::time::SystemTime;

use log::LevelFilter;

pub fn setup_logger(level: LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        // .chain(std::io::stdout())
        .chain(fern::log_file("logs/output.log")?)
        .apply()?;

    Ok(())
}

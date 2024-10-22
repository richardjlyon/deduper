mod error;
pub mod image;

use log::LevelFilter;

pub fn setup_logger(level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = fern::log_file("output.log")?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .chain(log_file)
        .apply()?;

    Ok(())
}

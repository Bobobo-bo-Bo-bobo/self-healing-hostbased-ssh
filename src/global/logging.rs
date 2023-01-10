pub fn init(level: log::LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, msg, record| {
            out.finish(format_args!(
                "{:<6}: {} {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%z"),
                msg
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TerminalMode};

pub(crate) fn set_log_config(quite: bool, verbosity: u8) {
    let log_level = if quite {
        LevelFilter::Error
    } else {
        match verbosity {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            3 => LevelFilter::Trace,
            _ => unreachable!(),
        }
    };

    let config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Debug)
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_level_padding(simplelog::LevelPadding::Left)
        .add_filter_allow("dotagents".into())
        .build();

    simplelog::TermLogger::init(log_level, config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();
}

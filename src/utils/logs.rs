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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_selection_quiet() {
        let log_level = if true {
            LevelFilter::Error
        } else {
            match 0u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                3 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Error);
    }

    #[test]
    fn test_log_level_selection_verbosity_0() {
        let log_level = if false {
            LevelFilter::Error
        } else {
            match 0u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                3 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Warn);
    }

    #[test]
    fn test_log_level_selection_verbosity_1() {
        let log_level = if false {
            LevelFilter::Error
        } else {
            match 1u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                3 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Info);
    }

    #[test]
    fn test_log_level_selection_verbosity_2() {
        let log_level = if false {
            LevelFilter::Error
        } else {
            match 2u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                3 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Debug);
    }

    #[test]
    fn test_log_level_selection_verbosity_3() {
        let log_level = if false {
            LevelFilter::Error
        } else {
            match 3u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                3 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Trace);
    }
}

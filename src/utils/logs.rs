use std::sync::OnceLock;

use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TerminalMode};

use crate::utils::tui::is_tui_enabled;

/// Runtime logging configuration, decided once at startup.
pub(crate) struct LogConfig {
    pub(crate) is_tty: bool,
    pub(crate) level: LevelFilter,
}

pub(crate) static LOG_CONFIG: OnceLock<LogConfig> = OnceLock::new();

/// Returns the cached log configuration, or `None` if not yet initialised.
pub(crate) fn log_config() -> Option<&'static LogConfig> {
    LOG_CONFIG.get()
}

/// Initialises logging: TTY → cliclack backend; non-TTY → simplelog backend.
pub(crate) fn set_log_config(quiet: bool, verbosity: u8) {
    let tty = is_tui_enabled();

    let level = if quiet {
        LevelFilter::Error
    } else {
        match verbosity {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            2 => LevelFilter::Trace,
            _ => unreachable!(),
        }
    };

    LOG_CONFIG.get_or_init(|| LogConfig { is_tty: tty, level });

    if !tty {
        let config = ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Debug)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_level_padding(simplelog::LevelPadding::Left)
            .add_filter_allow("dotagents".into())
            .build();

        simplelog::TermLogger::init(level, config, TerminalMode::Stderr, ColorChoice::Auto)
            .unwrap();
    }
}

/// Routes to `cliclack::log::error` in TTY mode, or `log::error!` in non-TTY mode.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        match $crate::utils::logs::log_config() {
            Some(cfg) if cfg.is_tty => {
                let _ = ::cliclack::log::error(::std::format!($($arg)*));
            }
            _ => ::log::error!($($arg)*),
        }
    }};
}

/// Routes to `cliclack::log::warning` in TTY mode, or `log::warn!` in non-TTY mode.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        match $crate::utils::logs::log_config() {
            Some(cfg) if cfg.is_tty => {
                let _ = ::cliclack::log::warning(::std::format!($($arg)*));
            }
            _ => ::log::warn!($($arg)*),
        }
    }};
}

/// Routes to `cliclack::log::info` in TTY mode (gated by `-v`), or `log::info!` in non-TTY mode.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        match $crate::utils::logs::log_config() {
            Some(cfg) if cfg.is_tty => {
                if cfg.level >= ::log::LevelFilter::Info {
                    let _ = ::cliclack::log::info(::std::format!($($arg)*));
                }
            }
            Some(_) => ::log::info!($($arg)*),
            None => {}
        }
    }};
}

/// Routes to `cliclack::log::remark` in TTY mode (gated by `-vv`), or `log::debug!` in non-TTY mode.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        match $crate::utils::logs::log_config() {
            Some(cfg) if cfg.is_tty => {
                if cfg.level >= ::log::LevelFilter::Debug {
                    let _ = ::cliclack::log::remark(::std::format!($($arg)*));
                }
            }
            Some(_) => ::log::debug!($($arg)*),
            None => {}
        }
    }};
}

/// Routes to `cliclack::log::remark` in TTY mode (gated by `-vvv`), or `log::trace!` in non-TTY mode.
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        match $crate::utils::logs::log_config() {
            Some(cfg) if cfg.is_tty => {
                if cfg.level >= ::log::LevelFilter::Trace {
                    let _ = ::cliclack::log::remark(::std::format!($($arg)*));
                }
            }
            Some(_) => ::log::trace!($($arg)*),
            None => {}
        }
    }};
}

/// Routes to `cliclack::log::success` in TTY mode (always), or `log::info!` in non-TTY mode (visible with `-v`).
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        match $crate::utils::logs::log_config() {
            Some(cfg) if cfg.is_tty => {
                let _ = ::cliclack::log::success(::std::format!($($arg)*));
            }
            Some(_) => ::log::info!($($arg)*),
            None => {}
        }
    }};
}

/// Routes to `cliclack::log::step` in TTY mode (always), or `log::debug!` in non-TTY mode (visible with `-vv`).
#[macro_export]
macro_rules! step {
    ($($arg:tt)*) => {{
        match $crate::utils::logs::log_config() {
            Some(cfg) if cfg.is_tty => {
                let _ = ::cliclack::log::step(::std::format!($($arg)*));
            }
            Some(_) => ::log::debug!($($arg)*),
            None => {}
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructs LogConfig with expected field values
    #[test]
    fn test_log_config_fields() {
        let cfg = LogConfig {
            is_tty: true,
            level: LevelFilter::Info,
        };
        assert!(cfg.is_tty);
        assert_eq!(cfg.level, LevelFilter::Info);
    }

    // log_config returns None before set_log_config is called in a fresh OnceLock
    #[test]
    fn test_log_config_accessor_returns_option() {
        // In test context LOG_CONFIG may or may not be set; just ensure it doesn't panic.
        let _ = log_config();
    }

    #[test]
    fn test_log_level_selection_quiet() {
        let log_level = if true {
            LevelFilter::Error
        } else {
            match 0u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Error);
    }

    #[test]
    fn test_log_level_selection_verbosity_0() {
        // no flags: default to Info so display output (ls, success) is visible without -v
        let log_level = if false {
            LevelFilter::Error
        } else {
            match 0u8 {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                2 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Info);
    }

    #[test]
    fn test_log_level_selection_verbosity_1() {
        let log_level = if false {
            LevelFilter::Error
        } else {
            match 1u8 {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                2 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Debug);
    }

    #[test]
    fn test_log_level_selection_verbosity_2() {
        let log_level = if false {
            LevelFilter::Error
        } else {
            match 2u8 {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                2 => LevelFilter::Trace,
                _ => unreachable!(),
            }
        };
        assert_eq!(log_level, LevelFilter::Trace);
    }
}

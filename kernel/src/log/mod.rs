use buflog::BufLogger;

mod buflog;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl core::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Warn => write!(f, "Warn"),
            LogLevel::Error => write!(f, "Error"),
        }
    }
}

pub trait LogWrite {
    fn write_str(&self, s: &str);
}

static LOGGER: BufLogger = BufLogger::new();

pub fn logger() -> &'static BufLogger {
    &LOGGER
}

#[macro_export]
macro_rules! print {
    ($($args: tt)*) => {
        $crate::log::logger().write(format_args!($($args)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::log::logger().write(format_args!("\n"))
    };
    ($($args: tt)*) => {
        $crate::log::logger().write(format_args_nl!($($args)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($args: tt)*) => {
        $crate::log::logger().log($crate::log::LogLevel::Info, format_args_nl!($($args)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($args: tt)*) => {
        $crate::log::logger().log($crate::log::LogLevel::Warn, format_args_nl!($($args)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($args: tt)*) => {
        $crate::log::logger().log($crate::log::LogLevel::Error, format_args_nl!($($args)*))
    };
}

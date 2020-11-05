use std::io;
use crate::utils;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LogLevel {
    All,
    Warn,
    Info,
    Debug,
    Trace,
    None
}

/* The value produced by this function must be compatible with LogLevel ordering */
fn num_level(log_level: LogLevel) -> u8 {
    match log_level {
        LogLevel::All   => 0,
        LogLevel::Trace => 1,
        LogLevel::Debug => 2,
        LogLevel::Info  => 3,
        LogLevel::Warn  => 4,
        LogLevel::None  => 5,
    }
}


impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        num_level(*self).partial_cmp(&num_level(*other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        num_level(*self).cmp(&num_level(*other))
    }
}

pub trait Logger {

    fn allows(&self, log_level: LogLevel) -> bool;

    fn writer(&mut self, log_level: LogLevel) -> Option<&mut dyn io::Write>;
}

//fn level_allows_tracing(allowed_level: LogLevel, log_level: LogLevel) -> bool {
    //assert!(log_level != LogLevel::All && log_level != LogLevel::None);

    //allowed_level <= log_level && allowed_level != LogLevel::None
//}


/* AnyLogger */

pub struct AnyLogger<W> {
    level:  LogLevel,
    writer: W
}

impl<W: io::Write> Logger for AnyLogger<W> {

    fn allows(&self, log_level: LogLevel) -> bool {
        assert!(log_level != LogLevel::All && log_level != LogLevel::None);

        self.level <= log_level && self.level != LogLevel::None
    }

    fn writer(&mut self, log_level: LogLevel) -> Option<&mut dyn io::Write> {
        utils::some_if(self.allows(log_level), &mut self.writer)
        //if self.allows(log_level) {
            //Some(&mut self.writer)
        //}
        //else {
            //None
        //}
    }
}

pub fn log_to<W: io::Write>(writer: W, log_level: LogLevel) -> AnyLogger<W> {
    AnyLogger::<W> {
        level: log_level,
        writer
    }
}

#[allow(dead_code)]
pub fn log_nothing() -> AnyLogger<io::Sink> {
    log_to(io::sink(), LogLevel::None)
}

//pub struct ErasedLogger {
    //erased_writer: Box<dyn io::Write>
//}

//pub fn erase<W>(writer: W) -> ErasedLogger
    //where W: Into<Box<dyn io::Write>>
//{
    //ErasedLogger {
        //erased_writer: writer.into()
    //}
//}

//trait Logger {

//}

#[macro_export]
macro_rules! log {
    ($logger:expr, $log_level:expr, $fmt:literal $(, $arg:expr)*) => {
        $logger.writer($log_level)
               .ok_or(std::io::Error::new(std::io::ErrorKind::WriteZero, "Insufficient logging level"))
               .and_then(|w| writeln!(w, $fmt $(, $arg)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Warn, $fmt $(, $arg)*)
    }
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Info, $fmt $(, $arg)*)
    }
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Debug, $fmt $(, $arg)*)
    }
}

#[macro_export]
macro_rules! trace {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Trace, $fmt $(, $arg)*)
    }
}

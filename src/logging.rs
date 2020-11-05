use std::io;
use crate::utils;
use std::fs::File;
use std::path::Path;
use core::cmp::Ordering;

#[derive(PartialEq, /*Eq,*/ Clone, Copy)]
pub enum LogLevel {
    All,
    Warn,
    Info,
    Debug,
    Trace,
    None
}

/* The value produced by this function must be compatible with LogLevel ordering */
fn num_level(log_level: LogLevel) -> Option<u8> {
    match log_level {
        LogLevel::All   => None,//0,
        LogLevel::Trace => Some(1),
        LogLevel::Debug => Some(2),
        LogLevel::Info  => Some(3),
        LogLevel::Warn  => Some(4),
        LogLevel::None  => None,//5,
    }
}


impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (*self, *other) {
            (LogLevel::All, _             ) => None,
            (_,             LogLevel::All ) => None,
            (LogLevel::None, _            ) => None,
            (_,             LogLevel::None) => None,

            _ => {
                let self_num  = num_level(*self).unwrap();
                let other_num = num_level(*other).unwrap();
                self_num.partial_cmp(&other_num)
            }
        }


        //num_level(*self).partial_cmp(&num_level(*other))
        //match (*self, *other) {
            //(LogLevel::All, LogLevel::All) => Some(Ordering::Equal),
            //(LogLevel::All, _)             => Some(Ordering::Less),

            //(LogLevel::Warn, LogLevel::All) => Some(Ordering::Equal),
            //(LogLevel::All, _)             => Some(Ordering::Less),
        //}
    }
}

//impl Ord for LogLevel {
    //fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        //num_level(*self).cmp(&num_level(*other))
    //}
//}

impl LogLevel {
    pub fn lower(&self) -> LogLevel {
        match *self {
            LogLevel::All   => LogLevel::Warn,
            LogLevel::Warn  => LogLevel::Info,
            LogLevel::Info  => LogLevel::Debug,
            LogLevel::Debug => LogLevel::Trace,
            LogLevel::Trace => LogLevel::None,
            LogLevel::None  => LogLevel::None,
        }
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
        //assert!(log_level != LogLevel::All && log_level != LogLevel::None);
        if self.level == LogLevel::None || log_level == LogLevel::None {
            false
        }
        else if self.level == LogLevel::All || log_level == LogLevel::All {
            true
        }
        else {
            self.level <= log_level
        }

        //match (self.level, log_level) {
            //(LogLevel::None, _             ) => false,
            //(_             , LogLevel::None) => false,

            //(LogLevel::All, _            ) => true,
            //(_            , LogLevel::All) => true,

            //_ =>
                //self.level <= log_level && self.level != LogLevel::None
        //}
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

pub fn log_to_file(file_path: &Path, log_level: LogLevel) -> Result<AnyLogger<File>, io::Error> {
    File::create(file_path)
        .map(|file| log_to(file, log_level))
}

pub fn ignore_all() -> AnyLogger<io::Sink> {
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

//#[macro_export]
macro_rules! log {
    ($logger:expr, $log_level:expr, $fmt:literal $(, $arg:expr)*) => {
        crate::logging::Logger::writer($logger, $log_level)
            .ok_or(std::io::Error::new(std::io::ErrorKind::WriteZero, "Insufficient logging level"))
            .and_then(|w| writeln!(w, $fmt $(, $arg)*))
    }
}

//#[macro_export]
macro_rules! warn {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Warn, $fmt $(, $arg)*)
    }
}

//#[macro_export]
macro_rules! info {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Info, $fmt $(, $arg)*)
    }
}

//#[macro_export]
macro_rules! debug {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Debug, $fmt $(, $arg)*)
    }
}

//#[macro_export]
macro_rules! trace {
    ($logger:expr, $fmt:literal $(, $arg:expr)*) => {
        log!($logger, crate::logging::LogLevel::Trace, $fmt $(, $arg)*)
    }
}

//#[macro_export]
macro_rules! log_nol {
    ($logger:expr, $log_level:expr, $fmt:literal $(, $arg:expr)*) => {
        crate::logging::Logger::writer($logger, $log_level)
            .ok_or(std::io::Error::new(std::io::ErrorKind::WriteZero, "Insufficient logging level"))
            .and_then(|w| write!(w, $fmt $(, $arg)*))
    }
}

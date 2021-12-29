//! Provides a basic and default log message formatter.

use std::fmt::Write;

use chrono::prelude::*;

use crate::{
    formatter::{FmtExtraInfo, Formatter},
    Record, Result, StringBuf,
};

/// A basic and default log message formatter.
///
/// The log message formatted by it looks like this:
/// `[2021-12-23 01:23:45.067] [info] log message`.
pub struct BasicFormatter {
    local_time_cacher: spin::Mutex<LocalTimeCacher>,
}

impl BasicFormatter {
    /// Constructs a [`BasicFormatter`].
    pub fn new() -> BasicFormatter {
        BasicFormatter {
            local_time_cacher: spin::Mutex::new(LocalTimeCacher::new()),
        }
    }
}

impl Formatter for BasicFormatter {
    fn format(&self, record: &Record, dest: &mut StringBuf) -> Result<FmtExtraInfo> {
        let time = self.local_time_cacher.lock().get(record.time());

        write!(
            dest,
            "[{}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}] [",
            // `time.format("%Y-%m-%d %H:%M:%S.%3f")` is slower than this way
            time.year,
            time.month,
            time.day,
            time.hour,
            time.minute,
            time.second,
            time.millisecond,
        )?;

        if let Some(logger_name) = record.logger_name() {
            write!(dest, "{}] [", logger_name)?;
        }

        let style_range_begin = dest.len();

        write!(dest, "{}", record.level())?;

        let style_range_end = dest.len();

        if let Some(srcloc) = record.source_location() {
            write!(dest, "] [{}:{}", srcloc.file_name(), srcloc.line())?;
        }

        write!(dest, "] {}", record.payload())?;

        Ok(FmtExtraInfo {
            style_range: Some(style_range_begin..style_range_end),
        })
    }
}

impl Default for BasicFormatter {
    fn default() -> BasicFormatter {
        BasicFormatter::new()
    }
}

#[derive(Clone, Default)]
struct LocalTimeCacher {
    cache: Option<LocalTimeCache>,
}

impl LocalTimeCacher {
    fn new() -> LocalTimeCacher {
        LocalTimeCacher::default()
    }

    fn cache(utc_time: &DateTime<Utc>) -> LocalTimeCache {
        LocalTimeCache {
            last_secs: utc_time.timestamp(),
            local_time: Into::<DateTime<Local>>::into(*utc_time).into(),
        }
    }

    fn get(&mut self, utc_time: &DateTime<Utc>) -> Time {
        match &mut self.cache {
            None => self.cache = Some(Self::cache(utc_time)),
            Some(cache) => {
                let secs = utc_time.timestamp();

                if cache.last_secs != secs {
                    *cache = Self::cache(utc_time);
                } else {
                    // update nanosecond

                    // `chrono::Timelike::with_nanosecond` is slower than this way
                    cache
                        .local_time
                        .set_millisecond_from_nanosecond(utc_time.nanosecond());
                }
            }
        }

        self.cache.as_ref().unwrap().local_time.clone()
    }
}

#[derive(Clone)]
struct LocalTimeCache {
    last_secs: i64,
    local_time: Time,
}

#[derive(Clone)]
struct Time {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
}

impl<T> From<DateTime<T>> for Time
where
    T: TimeZone,
{
    fn from(date_time: DateTime<T>) -> Time {
        Time {
            year: date_time.year(),
            month: date_time.month(),
            day: date_time.day(),
            hour: date_time.hour(),
            minute: date_time.minute(),
            second: date_time.second(),
            millisecond: Self::nanosecond_to_millisecond(date_time.nanosecond()),
        }
    }
}

impl Time {
    fn set_millisecond_from_nanosecond(&mut self, nanosecond: u32) {
        self.millisecond = Self::nanosecond_to_millisecond(nanosecond);
    }

    fn nanosecond_to_millisecond(nanosecond: u32) -> u32 {
        nanosecond % 1_000_000_000 / 1_000_000
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Level;

    #[test]
    fn format() {
        let record = Record::new(Level::Warn, "test log content");
        let mut buf = StringBuf::new();
        let extra_info = BasicFormatter::new().format(&record, &mut buf).unwrap();

        assert_eq!(
            format!(
                "[{}] [warn] test log content",
                Into::<DateTime::<Local>>::into(record.time().clone())
                    .format("%Y-%m-%d %H:%M:%S.%3f")
            ),
            buf
        );
        assert_eq!(Some(27..31), extra_info.style_range());
    }
}

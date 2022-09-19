use std::fmt::Write;

use crate::{
    formatter::pattern_formatter::{Pattern, PatternContext},
    Error, Record, StringBuf,
};

/// A pattern that writes an EOL character into the output.
///
/// # Implementation
///
/// On non-Windows systems, this pattern writes a `\n` to the output.
///
/// On Windows, this pattern writes a `\r\n` to the output.
#[derive(Copy, Clone, Debug, Default)]
pub struct Eol;

impl Eol {
    /// Create a new `Eol` pattern.
    pub fn new() -> Self {
        Self
    }
}

impl Pattern for Eol {
    fn format(
        &self,
        _record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        dest.write_str(crate::EOL).map_err(Error::FormatRecord)
    }
}

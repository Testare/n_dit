use std::io::{Error, Write};

use crate::prelude::*;
use flexi_logger::{DeferredNow, LogSpecification, LoggerHandle};
use log::Record;

#[derive(Deref, DerefMut, Resource)]
pub struct Log(pub LoggerHandle);

impl std::fmt::Debug for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Log")
    }
}

impl Log {
    pub fn push_spec(&mut self, s: &str) {
        self.0.push_temp_spec(LogSpecification::parse(s).unwrap());
    }
}

pub fn std_log_fmt(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record<'_>,
) -> Result<(), Error> {
    write!(
        w,
        "{} {:7} {}: {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        format!("[{}]", record.level()),
        record.module_path().unwrap_or("<unnamed>"),
        record.args()
    )
}

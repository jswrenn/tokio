//! Task dumps.

use std::fmt;

/// (WIP) Request a task dump.
#[derive(Debug)]
pub struct TaskDumpRequest {

}

/// (WIP) A task dump.
pub struct TaskDump {}

impl TaskDumpRequest {
    /// Construct a new `TaskDumpRequest`.
    pub fn new() -> Self {
        todo!()
    }

    /// boop
    pub fn request() -> TaskDump {
        todo!()
    }
}

impl TaskDump {}

impl fmt::Debug for TaskDump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
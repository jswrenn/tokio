//! Task dumps.

use std::fmt;

use crate::runtime::Handle;
use serde::{self, Serialize};

/// (WIP) A task dump.
pub struct TaskDump {

}

#[derive(Serialize)]
#[serde(tag = "flavor")]
enum Runtime {


}


impl TaskDump {
    fn new(handle: &Handle) -> Self {
        
        TaskDump {}
    }
}

impl fmt::Debug for TaskDump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
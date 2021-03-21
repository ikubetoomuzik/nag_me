//! module containing the builder for our task struct.

use chrono::{DateTime, Local};

//local import
use super::{TaskImportance, TaskStatus};

/// TODO:
pub struct TaskBuilder {
    /// Optional name for the task.
    pub name: Option<String>,
    /// Optional deadline for the task.
    pub deadline: Option<DateTime<Local>>,
    /// Optional importance for the task.
    pub importance: Option<TaskImportance>,
    /// Optional status for the task.
    pub status: Option<TaskStatus>,
    /// Optional subtask list for the task.
    pub subtasks: Vec<TaskBuilder>,
}

impl TaskBuilder {
    /// Generate a blank TaskBuilder.
    pub fn new() -> Self {
        Self {
            name: None,
            deadline: None,
            importance: None,
            status: None,
            subtasks: Vec::new(),
        }
    }

    /// Set the name.
    pub fn name(self, val: &str) -> Self {
        Self {
            name: Some(val.to_string()),
            ..self
        }
    }

    /// Set the deadline.
    pub fn deadline(self, val: DateTime<Local>) -> Self {
        Self {
            deadline: Some(val),
            ..self
        }
    }

    /// Set the importance.
    pub fn importance(self, val: TaskImportance) -> Self {
        Self {
            importance: Some(val),
            ..self
        }
    }

    /// Set the status.
    pub fn status(self, val: TaskStatus) -> Self {
        Self {
            status: Some(val),
            ..self
        }
    }

    /// Add a subtask builder.
    pub fn add_subtask(mut self, val: TaskBuilder) -> Self {
        self.subtasks.push(val);
        self
    }
}

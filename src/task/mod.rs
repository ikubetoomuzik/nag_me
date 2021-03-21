#![allow(dead_code)]
//! Task structs.
//! By: Curtis Jones
//! Started on: March 20, 2021

// External imports.
use chrono::prelude::*;
use chrono::Duration;
use uuid::Uuid;

// std lib imports.
use std::error::Error;
use std::fmt;
use std::slice::{Iter, IterMut};

// modules
pub mod builder;
pub mod progress;

// local crate imports;
pub use builder::TaskBuilder;
use progress::{Completion, ProgressNote};

/// Main struct of the code, defines how we understand Tasks.
#[derive(Debug)]
pub struct Task {
    id: Uuid,
    name: String,
    deadline: Option<DateTime<Local>>,
    importance: TaskImportance,
    status: TaskStatus,
    subtasks: Vec<Task>,
    notes: Vec<ProgressNote>,
}

impl Default for Task {
    fn default() -> Self {
        Self::new(TaskBuilder::new())
    }
}

impl Task {
    /// # Summary
    /// Basic constructor for a Task.
    /// No deadline.
    ///
    /// # Parameters
    /// * name => String for the name of the task.
    pub fn new(mut builder: TaskBuilder) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: builder.name.unwrap_or(String::from("new task...")),
            deadline: builder.deadline,
            importance: builder.importance.unwrap_or(TaskImportance::Normal),
            status: builder.status.unwrap_or(TaskStatus::InProgress),
            subtasks: builder
                .subtasks
                .drain(..)
                .map(|subtask_builder| Self::new(subtask_builder))
                .collect(),
            notes: Vec::new(),
        }
    }

    // Getters
    /// Get the id of a task.
    pub fn id(&self) -> Uuid {
        self.id
    }
    /// Get the name of a task.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Get the deadline of a task.
    pub fn deadline(&self) -> Option<DateTime<Local>> {
        self.deadline
    }
    /// Get the importance of a task.
    pub fn importance(&self) -> TaskImportance {
        self.importance
    }
    /// Get the status of a Task.
    pub fn status(&self) -> TaskStatus {
        self.status
    }

    /// Get the percentage of Task that is completed.
    pub fn completion(&self) -> Completion {
        if self.status == TaskStatus::Completed {
            // if the task is marked complete we return a 100% reading.
            Completion::full()
        } else {
            // return that notes completion percentage averaged with subtasks.
            let sub_len = self.subtasks.len() + 1;
            Completion::new(
                self.subtasks
                    .iter()
                    .fold(self.completion_notes_only().val(), |acc, subtask| {
                        acc + subtask.completion().val()
                    })
                    / sub_len as i32,
            )
        }
    }

    fn completion_notes_only(&self) -> Completion {
        // get the completion markers from the notes and sum them.
        self.notes.iter().fold(Completion::zero(), |acc, note| {
            if let Some(perc) = note.completed {
                acc + perc
            } else {
                acc
            }
        })
    }

    ///docs
    pub fn completion_breakdown(&self) -> (usize, Vec<(&str, Completion)>) {
        let val_len = self.subtasks.len() + 1;
        (
            val_len,
            self.subtasks_iter().fold(
                vec![("notes_only", self.completion_notes_only())],
                |mut acc, subtask| {
                    acc.push((subtask.name(), subtask.completion()));
                    acc
                },
            ),
        )
    }

    /// Iter over subtasks.
    pub fn subtasks_iter(&self) -> Iter<'_, Task> {
        self.subtasks.iter()
    }
    /// Iter over subtasks.
    pub fn subtasks_iter_mut(&mut self) -> IterMut<'_, Task> {
        self.subtasks.iter_mut()
    }

    // Set / Remove functions...

    /// Function to pause a running task.
    pub fn pause(&mut self) -> Result<(), Box<dyn Error>> {
        if self.status == TaskStatus::InProgress {
            for task in self.subtasks.iter_mut() {
                task.pause()?;
            }
            self.status = TaskStatus::OnHold;
            Ok(())
        } else {
            Err(Box::new(TaskError::TaskStatusError(format!(
                "Task {} is not currently in progress!",
                self.name
            ))))
        }
    }

    /// Function to resume a paused task.
    pub fn resume(&mut self) -> Result<(), Box<dyn Error>> {
        match self.status {
            TaskStatus::OnHold => {
                for task in self
                    .subtasks
                    .iter_mut()
                    // this makes sure that only the first explicit call of resume checks for
                    // completed.
                    .filter(|task| task.status != TaskStatus::Completed)
                {
                    task.resume()?;
                }
                self.status = TaskStatus::InProgress;
                Ok(())
            }
            TaskStatus::InProgress => Ok(()),
            TaskStatus::Completed => Err(Box::new(TaskError::TaskStatusError(format!(
                "Task {} is already completed!",
                self.name
            )))),
        }
    }

    /// Function to mark complete a currently active task.
    pub fn complete(&mut self) -> Result<(), Box<dyn Error>> {
        if self.status != TaskStatus::Completed {
            for task in self
                .subtasks
                .iter_mut()
                // this makes sure that only the first explicit call of resume checks for
                // completed.
                .filter(|task| task.status != TaskStatus::Completed)
            {
                task.complete()?;
            }
            self.status = TaskStatus::Completed;
            Ok(())
        } else {
            Err(Box::new(TaskError::TaskStatusError(format!(
                "Task {} is already complete!",
                self.name
            ))))
        }
    }

    /// Function to restart a completed task.
    pub fn restart(&mut self) -> Result<(), Box<dyn Error>> {
        if self.status == TaskStatus::Completed {
            for task in self.subtasks.iter_mut() {
                task.restart()?;
            }
            // reset the notes completion status, but keep the notes.
            self.notes
                .iter_mut()
                .for_each(|note| note.reset_completion());
            self.status = TaskStatus::InProgress;
            Ok(())
        } else {
            Err(Box::new(TaskError::TaskStatusError(format!(
                "Task {} has not been completed yet!",
                self.name
            ))))
        }
    }

    /// Function to reset a task to nothing, 0 notes.
    pub fn reset(&mut self) {
        for task in self.subtasks.iter_mut() {
            task.reset();
        }
        // clear all values but don't do the realloc, because the call is recursive
        self.notes.clear();
        self.status = TaskStatus::InProgress;
    }

    /// Change the importance of the current task.
    /// Return the prev value if it was different, else return none.
    pub fn change_importance(&mut self, new: TaskImportance) -> Option<TaskImportance> {
        if self.importance != new {
            let old = self.importance;
            self.importance = new;
            Some(old)
        } else {
            None
        }
    }

    /// Change the deadline of the current task.
    /// Return the prev value if it was different, else return none.
    pub fn change_deadline(&mut self, deadline: DateTime<Local>) -> Option<DateTime<Local>> {
        self.deadline.replace(deadline)
    }

    /// # Summary
    /// Extend the deadline of the current task.
    /// Return the prev value if it was different, else return none.
    ///
    /// # Parameters
    /// * duration: Duration to increase deadline by.
    ///
    /// # Return Val
    /// Returns an option containing old val if one was set and None if there was none.
    pub fn extend_deadline(&mut self, duration: Duration) -> Option<DateTime<Local>> {
        match self.deadline {
            Some(deadline) => self.deadline.replace(deadline + duration),
            None => None,
        }
    }

    /// Remove the deadline of a task.
    /// Return the prev value if it was different, else return none.
    pub fn remove_deadline(&mut self) -> Option<DateTime<Local>> {
        self.deadline.take()
    }

    /// Add a note and maybe some completion.
    pub fn add_note(&mut self, note: String, perc: Option<i32>) {
        self.notes.push(match perc {
            Some(p) => ProgressNote::with_completion(note, p),
            None => ProgressNote::new(note),
        });
    }
}

/// Enum representing importance of a task.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum TaskImportance {
    /// Lowest priority, basically just an idea.
    Casual,
    /// Default priority. Needs to get done but not desperate.
    Normal,
    /// Needs to be done and has a strict timeframe.
    Important,
    /// Needs to be done as soon as possible.
    Critical,
}

/// Enum representing importance of a task.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum TaskStatus {
    /// Currently being worked on.
    InProgress,
    /// Waiting to be turned back into active use.
    OnHold,
    /// Finished.
    Completed,
}

#[derive(Debug)]
/// Enum of different errors Tasks can produce.
pub enum TaskError {
    /// Error when trying to change/read task status.
    TaskStatusError(String),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::TaskStatusError(msg) => write!(f, "Invalid status: {}", msg),
        }
    }
}

impl Error for TaskError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn task_importance_ord() {
        assert!(TaskImportance::Critical > TaskImportance::Important);
    }

    #[test]
    fn task_build_new() {
        let deadline = Local::now() + Duration::days(10);
        let task = Task::new(
            TaskBuilder::new()
                .name("test")
                .deadline(deadline)
                .importance(TaskImportance::Critical)
                .status(TaskStatus::OnHold),
        );
        assert_eq!("test", task.name());
        assert_eq!(deadline, task.deadline().unwrap());
        assert_eq!(TaskImportance::Critical, task.importance());
        assert_eq!(TaskStatus::OnHold, task.status());
    }

    #[test]
    fn task_completed_basic() {
        let mut task = Task::default();
        task.add_note(String::new(), Some(40));
        assert_eq!(Completion::new(40), task.completion());
    }
}

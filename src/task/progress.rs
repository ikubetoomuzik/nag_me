//! Module containing progress notes.

use chrono::{DateTime, Local};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Completion percent.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Completion(i32);

impl Completion {
    /// Basic constructor.
    pub fn new(perc: i32) -> Self {
        assert!(perc >= 0 && perc <= 100);
        Self(perc)
    }
    /// Basic constructor for a completion starting at 0.
    pub const fn zero() -> Self {
        Self(0)
    }
    /// Basic constructor for a completion starting at 0.
    pub const fn full() -> Self {
        Self(100)
    }
    /// Check if the progress is full.
    pub fn is_complete(&self) -> bool {
        self.0 == 100
    }
    /// get val
    pub fn val(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for Completion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl Sub for Completion {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        let new_val = self.0 - other.0;
        if new_val > 0 {
            Self(new_val)
        } else {
            Self(0)
        }
    }
}

impl SubAssign for Completion {
    fn sub_assign(&mut self, other: Self) {
        let mut new_val = self.0 - other.0;
        if new_val < 0 {
            new_val = 0;
        }
        *self = Self(new_val);
    }
}

impl Add for Completion {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let new_val = self.0 + other.0;
        if new_val < 100 {
            Self(new_val)
        } else {
            Self(100)
        }
    }
}

impl AddAssign for Completion {
    fn add_assign(&mut self, other: Self) {
        let mut new_val = self.0 - other.0;
        if new_val > 100 {
            new_val = 100;
        }
        *self = Self(new_val);
    }
}

/// Struct to represent the notes within tasks to help with progress.
#[derive(Debug)]
pub struct ProgressNote {
    /// Notes about what was completed.
    pub note: String,
    /// Timestamp of note submission.
    pub timestamp: DateTime<Local>,
    /// Optional addition of a guess of how much of task was completed.
    /// Used to sum up completion of a task.
    pub completed: Option<Completion>,
}

impl ProgressNote {
    /// # Summary
    /// Basic constructor for a note.
    ///
    /// # Parameters
    /// * note: String => the note for this ProgressNote.
    pub fn new(note: String) -> Self {
        Self {
            note,
            timestamp: Local::now(),
            completed: None,
        }
    }

    /// # Summary
    /// Basic constructor for a note with a completion percent.
    ///
    /// # Parameters
    /// * note: String => the note for this ProgressNote.
    /// * perc: i32 => percent of task that was completed.
    pub fn with_completion(note: String, perc: i32) -> Self {
        Self {
            note,
            timestamp: Local::now(),
            completed: Some(Completion(perc)),
        }
    }

    /// # Summary
    /// Reset the completion to none.
    pub fn reset_completion(&mut self) {
        self.completed = None;
    }
}

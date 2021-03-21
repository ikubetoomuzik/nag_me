//! library for the nagger.
//!

use async_std::{
    channel,
    sync::{Arc, Mutex},
    task,
};
use chrono::{DateTime, Duration, Local};
use sorted_vec::SortedVec;
use std::cmp::Ordering;

/// docs
#[derive(Eq, PartialEq, Debug)]
pub struct Alarm {
    name: String,
    time: DateTime<Local>,
}

impl PartialOrd for Alarm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Alarm {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl Alarm {
    pub async fn activate(self) -> Self {
        let sleep_time = self.time - Local::now();
        task::sleep(sleep_time.to_std().unwrap()).await;
        self
    }
}

/// docs
pub struct Nagger {
    alarms: Arc<Mutex<SortedVec<Alarm>>>,
}

impl Nagger {
    /// init the Nagger
    pub async fn init(tx: channel::Sender<Alarm>) -> Self {
        let alarms: Arc<Mutex<SortedVec<Alarm>>> = Arc::new(Mutex::new(SortedVec::new()));
        // spawn the alarm running thread.
        let runner_alarms = Arc::clone(&alarms);
        task::spawn(Self::runner(runner_alarms, tx));
        // return our val with the handlers.
        Self { alarms }
    }

    /// add alarm to the queue.
    pub async fn add_alarm(&self, alarm: Alarm) {
        self.alarms.lock().await.insert(alarm);
    }

    /// delete alarm from the queue.
    pub async fn del_alarm(&self, name: &str) -> Option<Alarm> {
        let mut lock = self.alarms.lock().await;
        lock.iter()
            .position(|alarm| alarm.name == name)
            .map(|pos| lock.remove_index(pos))
    }

    async fn runner(alarms: Arc<Mutex<SortedVec<Alarm>>>, tx: channel::Sender<Alarm>) {
        loop {
            match alarms.lock().await.pop() {
                Some(alarm) => {
                    let alarm = alarm.activate().await;
                    if let Err(_) = tx.send(alarm).await {
                        break;
                    }
                }
                None => task::sleep(std::time::Duration::from_secs(1)).await,
            }
        }
    }
}

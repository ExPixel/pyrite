use std::{
    collections::BinaryHeap,
    time::{Duration, Instant},
};

use crossbeam::channel::{self, Receiver, RecvTimeoutError, Sender};
use parking_lot::RwLock;

fn run(receiver: Receiver<Work>) {
    let mut queue = BinaryHeap::<Work>::new();
    let mut next_work_item_time: Option<Instant> = None;

    tracing::debug!("startin working loop");
    'main_loop: loop {
        let now = Instant::now();

        let maybe_received = if let Some(time) = next_work_item_time.take() {
            let duration = time.saturating_duration_since(now);
            tracing::debug!("worker loop waiting {duration:?}");
            match receiver.recv_deadline(time) {
                Ok(message) => Some(message),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            tracing::debug!("worker loop waiting");
            match receiver.recv() {
                Ok(message) => Some(message),
                Err(_) => break,
            }
        };

        if let Some(received) = maybe_received {
            queue.push(received);
        }

        loop {
            if let Some(work) = queue.peek() {
                if work.time > now {
                    next_work_item_time = Some(work.time);
                    continue 'main_loop;
                }
            } else {
                continue 'main_loop;
            }

            let work = queue.pop().unwrap();
            (work.callback)();
        }
    }
    tracing::debug!("worker shutdown");
}

#[allow(dead_code)]
pub fn spawn<F>(callback: F)
where
    F: 'static + FnOnce() + Send,
{
    spawn_in_internal(Box::new(callback), Instant::now());
}

pub fn spawn_in<F>(callback: F, delay: Duration)
where
    F: 'static + FnOnce() + Send,
{
    spawn_in_internal(Box::new(callback), Instant::now() + delay);
}

#[allow(dead_code)]
pub fn spawn_at<F>(callback: F, time: Instant)
where
    F: 'static + FnOnce() + Send,
{
    spawn_in_internal(Box::new(callback), time);
}

fn spawn_in_internal(callback: Box<dyn Send + FnOnce()>, time: Instant) {
    let mut worker = WORKER.read();
    if worker.is_none() {
        parking_lot::RwLockReadGuard::unlocked(&mut worker, start);
    }

    worker
        .as_ref()
        .expect("no worker queue")
        .send(Work { time, callback })
        .expect("worker queue closed");
}

pub fn start() {
    let mut worker = WORKER.write();

    if worker.is_none() {
        let (send, recv) = channel::unbounded();
        std::thread::spawn(move || run(recv));
        let _ = worker.insert(send);
    }
}

#[allow(dead_code)]
pub fn stop() {
    let _ = WORKER.write().take();
}

static WORKER: RwLock<Option<Sender<Work>>> = parking_lot::const_rwlock(None);

struct Work {
    time: Instant,
    callback: Box<dyn 'static + FnOnce() + Send>,
}

impl PartialEq for Work {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Work {}

impl PartialOrd for Work {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time).map(|o| o.reverse())
    }
}

impl Ord for Work {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time).reverse()
    }
}

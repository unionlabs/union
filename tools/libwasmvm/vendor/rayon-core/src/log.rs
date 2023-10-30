//! Debug Logging
//!
//! To use in a debug build, set the env var `RAYON_LOG` as
//! described below.  In a release build, logs are compiled out by
//! default unless Rayon is built with `--cfg rayon_rs_log` (try
//! `RUSTFLAGS="--cfg rayon_rs_log"`).
//!
//! Note that logs are an internally debugging tool and their format
//! is considered unstable, as are the details of how to enable them.
//!
//! # Valid values for RAYON_LOG
//!
//! The `RAYON_LOG` variable can take on the following values:
//!
//! * `tail:<file>` -- dumps the last 10,000 events into the given file;
//!   useful for tracking down deadlocks
//! * `profile:<file>` -- dumps only those events needed to reconstruct how
//!   many workers are active at a given time
//! * `all:<file>` -- dumps every event to the file; useful for debugging

use crossbeam_channel::{self, Receiver, Sender};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};

/// True if logs are compiled in.
pub(super) const LOG_ENABLED: bool = cfg!(any(rayon_rs_log, debug_assertions));

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub(super) enum Event {
    /// Flushes events to disk, used to terminate benchmarking.
    Flush,

    /// Indicates that a worker thread started execution.
    ThreadStart {
        worker: usize,
        terminate_addr: usize,
    },

    /// Indicates that a worker thread started execution.
    ThreadTerminate { worker: usize },

    /// Indicates that a worker thread became idle, blocked on `latch_addr`.
    ThreadIdle { worker: usize, latch_addr: usize },

    /// Indicates that an idle worker thread found work to do, after
    /// yield rounds. It should no longer be considered idle.
    ThreadFoundWork { worker: usize, yields: u32 },

    /// Indicates that a worker blocked on a latch observed that it was set.
    ///
    /// Internal debugging event that does not affect the state
    /// machine.
    ThreadSawLatchSet { worker: usize, latch_addr: usize },

    /// Indicates that an idle worker is getting sleepy. `sleepy_counter` is the internal
    /// sleep state that we saw at the time.
    ThreadSleepy { worker: usize, jobs_counter: usize },

    /// Indicates that the thread's attempt to fall asleep was
    /// interrupted because the latch was set. (This is not, in and of
    /// itself, a change to the thread state.)
    ThreadSleepInterruptedByLatch { worker: usize, latch_addr: usize },

    /// Indicates that the thread's attempt to fall asleep was
    /// interrupted because a job was posted. (This is not, in and of
    /// itself, a change to the thread state.)
    ThreadSleepInterruptedByJob { worker: usize },

    /// Indicates that an idle worker has gone to sleep.
    ThreadSleeping { worker: usize, latch_addr: usize },

    /// Indicates that a sleeping worker has awoken.
    ThreadAwoken { worker: usize, latch_addr: usize },

    /// Indicates that the given worker thread was notified it should
    /// awaken.
    ThreadNotify { worker: usize },

    /// The given worker has pushed a job to its local deque.
    JobPushed { worker: usize },

    /// The given worker has popped a job from its local deque.
    JobPopped { worker: usize },

    /// The given worker has stolen a job from the deque of another.
    JobStolen { worker: usize, victim: usize },

    /// N jobs were injected into the global queue.
    JobsInjected { count: usize },

    /// A job was removed from the global queue.
    JobUninjected { worker: usize },

    /// A job was broadcasted to N threads.
    JobBroadcast { count: usize },

    /// When announcing a job, this was the value of the counters we observed.
    ///
    /// No effect on thread state, just a debugging event.
    JobThreadCounts {
        worker: usize,
        num_idle: u16,
        num_sleepers: u16,
    },
}

/// Handle to the logging thread, if any. You can use this to deliver
/// logs. You can also clone it freely.
#[derive(Clone)]
pub(super) struct Logger {
    sender: Option<Sender<Event>>,
}

impl Logger {
    pub(super) fn new(num_workers: usize) -> Logger {
        if !LOG_ENABLED {
            return Self::disabled();
        }

        // see the doc comment for the format
        let env_log = match env::var("RAYON_LOG") {
            Ok(s) => s,
            Err(_) => return Self::disabled(),
        };

        let (sender, receiver) = crossbeam_channel::unbounded();

        if let Some(filename) = env_log.strip_prefix("tail:") {
            let filename = filename.to_string();
            ::std::thread::spawn(move || {
                Self::tail_logger_thread(num_workers, filename, 10_000, receiver)
            });
        } else if env_log == "all" {
            ::std::thread::spawn(move || Self::all_logger_thread(num_workers, receiver));
        } else if let Some(filename) = env_log.strip_prefix("profile:") {
            let filename = filename.to_string();
            ::std::thread::spawn(move || {
                Self::profile_logger_thread(num_workers, filename, 10_000, receiver)
            });
        } else {
            panic!("RAYON_LOG should be 'tail:<file>' or 'profile:<file>'");
        }

        Logger {
            sender: Some(sender),
        }
    }

    fn disabled() -> Logger {
        Logger { sender: None }
    }

    #[inline]
    pub(super) fn log(&self, event: impl FnOnce() -> Event) {
        if !LOG_ENABLED {
            return;
        }

        if let Some(sender) = &self.sender {
            sender.send(event()).unwrap();
        }
    }

    fn profile_logger_thread(
        num_workers: usize,
        log_filename: String,
        capacity: usize,
        receiver: Receiver<Event>,
    ) {
        let file = File::create(&log_filename)
            .unwrap_or_else(|err| panic!("failed to open `{}`: {}", log_filename, err));

        let mut writer = BufWriter::new(file);
        let mut events = Vec::with_capacity(capacity);
        let mut state = SimulatorState::new(num_workers);
        let timeout = std::time::Duration::from_secs(30);

        loop {
            while let Ok(event) = receiver.recv_timeout(timeout) {
                if let Event::Flush = event {
                    break;
                }

                events.push(event);
                if events.len() == capacity {
                    break;
                }
            }

            for event in events.drain(..) {
                if state.simulate(&event) {
                    state.dump(&mut writer, &event).unwrap();
                }
            }

            writer.flush().unwrap();
        }
    }

    fn tail_logger_thread(
        num_workers: usize,
        log_filename: String,
        capacity: usize,
        receiver: Receiver<Event>,
    ) {
        let file = File::create(&log_filename)
            .unwrap_or_else(|err| panic!("failed to open `{}`: {}", log_filename, err));

        let mut writer = BufWriter::new(file);
        let mut events: VecDeque<Event> = VecDeque::with_capacity(capacity);
        let mut state = SimulatorState::new(num_workers);
        let timeout = std::time::Duration::from_secs(30);
        let mut skipped = false;

        loop {
            while let Ok(event) = receiver.recv_timeout(timeout) {
                if let Event::Flush = event {
                    // We ignore Flush events in tail mode --
                    // we're really just looking for
                    // deadlocks.
                    continue;
                } else {
                    if events.len() == capacity {
                        let event = events.pop_front().unwrap();
                        state.simulate(&event);
                        skipped = true;
                    }

                    events.push_back(event);
                }
            }

            if skipped {
                writeln!(writer, "...").unwrap();
                skipped = false;
            }

            for event in events.drain(..) {
                // In tail mode, we dump *all* events out, whether or
                // not they were 'interesting' to the state machine.
                state.simulate(&event);
                state.dump(&mut writer, &event).unwrap();
            }

            writer.flush().unwrap();
        }
    }

    fn all_logger_thread(num_workers: usize, receiver: Receiver<Event>) {
        let stderr = std::io::stderr();
        let mut state = SimulatorState::new(num_workers);

        for event in receiver {
            let mut writer = BufWriter::new(stderr.lock());
            state.simulate(&event);
            state.dump(&mut writer, &event).unwrap();
            writer.flush().unwrap();
        }
    }
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
enum State {
    Working,
    Idle,
    Notified,
    Sleeping,
    Terminated,
}

impl State {
    fn letter(&self) -> char {
        match self {
            State::Working => 'W',
            State::Idle => 'I',
            State::Notified => 'N',
            State::Sleeping => 'S',
            State::Terminated => 'T',
        }
    }
}

struct SimulatorState {
    local_queue_size: Vec<usize>,
    thread_states: Vec<State>,
    injector_size: usize,
}

impl SimulatorState {
    fn new(num_workers: usize) -> Self {
        Self {
            local_queue_size: (0..num_workers).map(|_| 0).collect(),
            thread_states: (0..num_workers).map(|_| State::Working).collect(),
            injector_size: 0,
        }
    }

    fn simulate(&mut self, event: &Event) -> bool {
        match *event {
            Event::ThreadIdle { worker, .. } => {
                assert_eq!(self.thread_states[worker], State::Working);
                self.thread_states[worker] = State::Idle;
                true
            }

            Event::ThreadStart { worker, .. } | Event::ThreadFoundWork { worker, .. } => {
                self.thread_states[worker] = State::Working;
                true
            }

            Event::ThreadTerminate { worker, .. } => {
                self.thread_states[worker] = State::Terminated;
                true
            }

            Event::ThreadSleeping { worker, .. } => {
                assert_eq!(self.thread_states[worker], State::Idle);
                self.thread_states[worker] = State::Sleeping;
                true
            }

            Event::ThreadAwoken { worker, .. } => {
                assert_eq!(self.thread_states[worker], State::Notified);
                self.thread_states[worker] = State::Idle;
                true
            }

            Event::JobPushed { worker } => {
                self.local_queue_size[worker] += 1;
                true
            }

            Event::JobPopped { worker } => {
                self.local_queue_size[worker] -= 1;
                true
            }

            Event::JobStolen { victim, .. } => {
                self.local_queue_size[victim] -= 1;
                true
            }

            Event::JobsInjected { count } => {
                self.injector_size += count;
                true
            }

            Event::JobUninjected { .. } => {
                self.injector_size -= 1;
                true
            }

            Event::ThreadNotify { worker } => {
                // Currently, this log event occurs while holding the
                // thread lock, so we should *always* see it before
                // the worker awakens.
                assert_eq!(self.thread_states[worker], State::Sleeping);
                self.thread_states[worker] = State::Notified;
                true
            }

            // remaining events are no-ops from pov of simulating the
            // thread state
            _ => false,
        }
    }

    fn dump(&mut self, w: &mut impl Write, event: &Event) -> io::Result<()> {
        let num_idle_threads = self
            .thread_states
            .iter()
            .filter(|s| **s == State::Idle)
            .count();

        let num_sleeping_threads = self
            .thread_states
            .iter()
            .filter(|s| **s == State::Sleeping)
            .count();

        let num_notified_threads = self
            .thread_states
            .iter()
            .filter(|s| **s == State::Notified)
            .count();

        let num_pending_jobs: usize = self.local_queue_size.iter().sum();

        write!(w, "{:2},", num_idle_threads)?;
        write!(w, "{:2},", num_sleeping_threads)?;
        write!(w, "{:2},", num_notified_threads)?;
        write!(w, "{:4},", num_pending_jobs)?;
        write!(w, "{:4},", self.injector_size)?;

        let event_str = format!("{:?}", event);
        write!(w, r#""{:60}","#, event_str)?;

        for ((i, state), queue_size) in (0..).zip(&self.thread_states).zip(&self.local_queue_size) {
            write!(w, " T{:02},{}", i, state.letter(),)?;

            if *queue_size > 0 {
                write!(w, ",{:03},", queue_size)?;
            } else {
                write!(w, ",   ,")?;
            }
        }

        writeln!(w)?;
        Ok(())
    }
}

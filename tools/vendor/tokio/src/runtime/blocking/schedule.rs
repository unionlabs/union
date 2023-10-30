use crate::runtime::task::{self, Task};

/// `task::Schedule` implementation that does nothing. This is unique to the
/// blocking scheduler as tasks scheduled are not really futures but blocking
/// operations.
///
/// We avoid storing the task by forgetting it in `bind` and re-materializing it
/// in `release.
pub(crate) struct NoopSchedule;

impl task::Schedule for NoopSchedule {
    fn release(&self, _task: &Task<Self>) -> Option<Task<Self>> {
        None
    }

    fn schedule(&self, _task: task::Notified<Self>) {
        unreachable!();
    }
}

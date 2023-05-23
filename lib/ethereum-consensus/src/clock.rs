//! A consensus clock
use crate::configs;
use crate::phase0 as presets;
use crate::primitives::{Epoch, Slot};
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn convert_timestamp_to_slot(timestamp: u64, genesis_time: u64, seconds_per_slot: u64) -> Slot {
    (timestamp - genesis_time) / seconds_per_slot
}

pub const MAINNET_GENESIS_TIME: u64 = 1606824023;
pub const SEPOLIA_GENESIS_TIME: u64 = 1655733600;
pub const GOERLI_GENESIS_TIME: u64 = 1616508000;

pub fn get_current_unix_time_in_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub trait TimeProvider {
    fn get_current_time(&self) -> u64;
}

#[derive(Clone)]
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn get_current_time(&self) -> u64 {
        get_current_unix_time_in_secs()
    }
}

#[derive(Clone)]
pub struct Clock<T: TimeProvider>(Arc<Inner<T>>);

impl<T: TimeProvider> Deref for Clock<T> {
    type Target = Inner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Inner<T: TimeProvider> {
    genesis_time: u64,
    seconds_per_slot: u64,
    slots_per_epoch: Slot,
    time_provider: T,
}

pub fn from_system_time(
    genesis_time: u64,
    seconds_per_slot: u64,
    slots_per_epoch: Slot,
) -> Clock<SystemTimeProvider> {
    let time_provider = SystemTimeProvider;
    Clock::new(
        genesis_time,
        seconds_per_slot,
        slots_per_epoch,
        time_provider,
    )
}

pub fn for_mainnet() -> Clock<SystemTimeProvider> {
    let genesis_time = MAINNET_GENESIS_TIME;
    let seconds_per_slot = configs::mainnet::SECONDS_PER_SLOT;
    let slots_per_epoch = presets::mainnet::SLOTS_PER_EPOCH;
    from_system_time(genesis_time, seconds_per_slot, slots_per_epoch)
}

pub fn for_sepolia() -> Clock<SystemTimeProvider> {
    let genesis_time = SEPOLIA_GENESIS_TIME;
    let seconds_per_slot = configs::sepolia::SECONDS_PER_SLOT;
    let slots_per_epoch = presets::mainnet::SLOTS_PER_EPOCH;
    from_system_time(genesis_time, seconds_per_slot, slots_per_epoch)
}

pub fn for_goerli() -> Clock<SystemTimeProvider> {
    let genesis_time = GOERLI_GENESIS_TIME;
    let seconds_per_slot = configs::goerli::SECONDS_PER_SLOT;
    let slots_per_epoch = presets::mainnet::SLOTS_PER_EPOCH;
    from_system_time(genesis_time, seconds_per_slot, slots_per_epoch)
}

impl<T: TimeProvider> Clock<T> {
    pub fn new(
        genesis_time: u64,
        seconds_per_slot: u64,
        slots_per_epoch: Slot,
        time_provider: T,
    ) -> Self {
        let inner = Inner {
            genesis_time,
            seconds_per_slot,
            slots_per_epoch,
            time_provider,
        };
        Self(Arc::new(inner))
    }

    fn get_current_time(&self) -> u64 {
        self.time_provider.get_current_time()
    }

    pub fn before_genesis(&self) -> bool {
        self.before_genesis_at(self.get_current_time())
    }

    #[inline]
    fn before_genesis_at(&self, current_time: u64) -> bool {
        current_time < self.genesis_time
    }

    // Return the current slot.
    // Panics if before genesis.
    pub fn current_slot(&self) -> Slot {
        self.slot_at_time(self.get_current_time())
    }

    #[inline]
    fn slot_at_time(&self, current_time: u64) -> Slot {
        let span = current_time - self.genesis_time;
        span / self.seconds_per_slot
    }

    // Return the current epoch.
    // Panics if before genesis.
    pub fn current_epoch(&self) -> Epoch {
        self.epoch_for(self.current_slot())
    }

    #[inline]
    pub fn epoch_for(&self, slot: Slot) -> Epoch {
        slot / self.slots_per_epoch
    }

    pub fn duration_until_next_slot(&self) -> Duration {
        let current_time = self.get_current_time();
        if self.before_genesis_at(current_time) {
            Duration::from_secs(self.genesis_time - current_time)
        } else {
            let current_slot = self.slot_at_time(current_time);
            let next_slot = current_slot + 1;
            let next_slot_in_secs = next_slot * self.seconds_per_slot + self.genesis_time;
            Duration::from_secs(next_slot_in_secs - current_time)
        }
    }
}

#[cfg(feature = "async")]
use tokio_stream::Stream;
#[cfg(feature = "async")]
impl<T: TimeProvider> Clock<T> {
    pub fn stream_slots(&self) -> impl Stream<Item = Slot> + '_ {
        async_stream::stream! {
            loop {
                let duration_until_next_slot = self.duration_until_next_slot();
                tokio::time::sleep(duration_until_next_slot).await;
                yield self.current_slot();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct Ticker {
        tick: Mutex<u64>,
        seconds_per_slot: u64,
    }

    impl Ticker {
        fn tick(&self) {
            let mut tick = self.tick.lock().unwrap();
            *tick += 1;
        }

        fn tick_slot(&self) {
            let mut tick = self.tick.lock().unwrap();
            *tick += self.seconds_per_slot;
        }
    }

    impl TimeProvider for Arc<Ticker> {
        fn get_current_time(&self) -> u64 {
            *self.tick.lock().unwrap()
        }
    }

    fn new_ticker(seconds_per_slot: u64) -> Arc<Ticker> {
        Arc::new(Ticker {
            tick: Mutex::new(0),
            seconds_per_slot,
        })
    }

    #[test]
    fn test_custom_time_provider() {
        let seconds_per_slot = 12;
        let time_provider = new_ticker(seconds_per_slot);
        let clock = Clock::new(0, seconds_per_slot, 32, time_provider.clone());
        assert_eq!(clock.duration_until_next_slot().as_secs(), 12);
        assert_eq!(clock.current_slot(), 0);
        time_provider.tick();
        assert_eq!(clock.duration_until_next_slot().as_secs(), 11);
        assert_eq!(clock.current_slot(), 0);
        for _ in 0..12 {
            time_provider.tick();
        }
        assert_eq!(clock.duration_until_next_slot().as_secs(), 11);
        assert_eq!(clock.current_slot(), 1);
    }

    #[test]
    fn test_before_genesis() {
        let seconds_per_slot = 12;
        let time_provider = new_ticker(seconds_per_slot);
        let clock = Clock::new(10, seconds_per_slot, 32, time_provider.clone());
        assert_eq!(clock.duration_until_next_slot().as_secs(), 10);
        time_provider.tick();
        assert_eq!(clock.duration_until_next_slot().as_secs(), 9);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_slot_stream() {
        use tokio_stream::StreamExt;

        let seconds_per_slot = 1;
        let time_provider = new_ticker(seconds_per_slot);
        let clock = Clock::new(0, seconds_per_slot, 12, time_provider.clone());
        let slot_stream = clock.stream_slots();

        tokio::pin!(slot_stream);

        let mut iter = 0;
        while let Some(slot) = slot_stream.next().await {
            dbg!(slot);
            time_provider.tick_slot();
            iter += 1;
            if iter > 1 {
                break;
            }
        }
    }
}

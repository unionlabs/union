use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint256;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("token bucket capacity must not be zero")]
    ZeroCapacity,
    #[error("token bucket refill rate must not be zero")]
    ZeroRefillRate,
    #[error("token bucket rate limit exceeded")]
    RateLimitExceeded,
}

#[cw_serde]
pub struct TokenBucket {
    pub capacity: Uint256,
    pub available: Uint256,
    pub refill_rate: Uint256,
    pub last_refill: Uint256,
}

impl TokenBucket {
    pub fn new(
        capacity: impl Into<Uint256>,
        refill_rate: impl Into<Uint256>,
        now: impl Into<Uint256>,
    ) -> Result<Self, Error> {
        let capacity = capacity.into();
        let refill_rate = refill_rate.into();
        let now = now.into();
        if capacity.is_zero() {
            return Err(Error::ZeroCapacity);
        }
        if refill_rate.is_zero() {
            return Err(Error::ZeroRefillRate);
        }
        Ok(Self {
            capacity,
            available: capacity,
            refill_rate,
            last_refill: now,
        })
    }

    pub fn refill(&mut self, now: Uint256) {
        if self.available >= self.capacity {
            self.last_refill = now;
            return;
        }
        let elapsed = now - self.last_refill;
        let to_refill = self.refill_rate.saturating_mul(elapsed);
        if !to_refill.is_zero() {
            self.available = self.capacity.min(self.available.saturating_add(to_refill));
            self.last_refill = now;
        }
    }

    pub fn rate_limit(&mut self, amount: Uint256, now: Uint256) -> Result<(), Error> {
        if amount.is_zero() {
            return Ok(());
        }
        self.refill(now);
        if self.available < amount {
            return Err(Error::RateLimitExceeded);
        }
        self.available -= amount;
        Ok(())
    }

    pub fn update(
        &mut self,
        capacity: Uint256,
        refill_rate: Uint256,
        reset: bool,
    ) -> Result<(), Error> {
        if capacity.is_zero() {
            return Err(Error::ZeroCapacity);
        }
        if refill_rate.is_zero() {
            return Err(Error::ZeroRefillRate);
        }
        self.capacity = capacity;
        self.refill_rate = refill_rate;
        if self.last_refill.is_zero() || reset {
            self.available = self.capacity;
        }
        Ok(())
    }
}

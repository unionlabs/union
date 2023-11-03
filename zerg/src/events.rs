use std::time::{SystemTime, UNIX_EPOCH};

use ucs01_relay_api::types::Ucs01TransferPacket;
use unionlabs::events::RecvPacket;
use uuid::Uuid;

/// A timestamped event originating from `chain_id`.
pub struct TimedEvent {
    pub execution_timestamp: u64,
    pub finalization_timestamp: u64,
    pub chain_id: String,
}

impl TimedEvent {
    pub fn new(chain_id: String, execution_timestamp: Option<u64>) -> Self {
        let finalization_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let execution_timestamp = match execution_timestamp {
            Some(timestamp) => timestamp,
            None => finalization_timestamp,
        };
        Self {
            chain_id,
            execution_timestamp,
            finalization_timestamp,
        }
    }
}

/// Event types tracked by Zerg when exporting to CSV
pub enum EventType {
    SendEvent(TimedEvent),
    ReceiveEvent(TimedEvent),
}

/// Event information recorded to the output CSV.
pub struct Event {
    pub sender: String,
    pub stamped_event: EventType,
    pub uuid: Uuid,
}

impl Event {
    /// Creates an `Event` originating from `chain_id` from the `SendPacket` event data.
    ///
    /// Constructs a unique ID from packet information in the form of:
    /// `<src_port>/<src_channel>/<sequence>`
    pub fn create_send_event(
        chain_id: String,
        uuid: Uuid,
        sender: String,
        execution_timestamp: Option<u64>,
    ) -> Event {
        let timed_event = TimedEvent::new(chain_id, execution_timestamp);

        Event {
            sender,
            stamped_event: EventType::SendEvent(timed_event),
            uuid,
        }
    }

    /// Creates an `Event` originating from `chain_id` from the `RecvPacket` event data.
    ///
    /// Constructs a unique ID from packet information in the form of:
    /// `<src_port>/<src_channel>/<sequence>`
    pub fn create_recv_event(
        chain_id: String,
        uuid: Uuid,
        e: RecvPacket,
        execution_timestamp: Option<u64>,
    ) -> Event {
        let transfer =
            Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();

        let timed_event = TimedEvent::new(chain_id, execution_timestamp);

        Event {
            uuid,
            sender: transfer.sender().to_string(),
            stamped_event: EventType::ReceiveEvent(timed_event),
        }
    }
}

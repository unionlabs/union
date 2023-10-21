use std::time::{SystemTime, UNIX_EPOCH};

use ucs01_relay_api::types::Ucs01TransferPacket;
use unionlabs::events::{RecvPacket, SendPacket};

/// A timestamped event originating from `chain_id`.
pub struct TimedEvent<T> {
    pub execution_timestamp: u64,
    pub finalization_timestamp: u64,
    pub event: T,
    pub chain_id: String,
}

impl<T> TimedEvent<T> {
    pub fn new(chain_id: String, event: T, execution_timestamp: Option<u64>) -> Self {
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
            event,
            execution_timestamp,
            finalization_timestamp,
        }
    }
}

/// Event types tracked by Zerg when exporting to CSV
pub enum EventType {
    SendEvent(TimedEvent<SendPacket>),
    ReceiveEvent(TimedEvent<RecvPacket>),
}

/// Event information recorded to the output CSV.
pub struct Event {
    pub sender: String,
    pub stamped_event: EventType,
    pub uuid: String,
}

impl Event {
    /// Creates an `Event` originating from `chain_id` from the `SendPacket` event data.
    ///
    /// Constructs a unique ID from packet information in the form of:
    /// `<src_port>/<src_channel>/<sequence>`
    pub fn create_send_event(
        chain_id: String,
        e: SendPacket,
        execution_timestamp: Option<u64>,
    ) -> Event {
        let transfer =
            Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();

        let uuid = format!(
            "{}/{}/{}",
            e.packet_src_port.clone(),
            e.packet_src_channel,
            e.packet_sequence
        );

        let timed_event = TimedEvent::new(chain_id, e, execution_timestamp);

        Event {
            sender: transfer.sender().to_string(),
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
        e: RecvPacket,
        execution_timestamp: Option<u64>,
    ) -> Event {
        let transfer =
            Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();

        let uuid = format!(
            "{}/{}/{}",
            e.packet_src_port.clone(),
            e.packet_src_channel,
            e.packet_sequence
        );

        let timed_event = TimedEvent::new(chain_id, e, execution_timestamp);

        Event {
            sender: transfer.sender().to_string(),
            stamped_event: EventType::ReceiveEvent(timed_event),
            uuid,
        }
    }
}

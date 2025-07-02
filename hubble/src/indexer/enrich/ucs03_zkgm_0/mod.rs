pub mod ack;
pub mod packet;
pub mod packet_ack;

pub struct PacketHash(pub [u8; 32]);

pub struct PacketPathHash(pub [u8; 32]);

impl PacketPathHash {
    fn to_0x_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

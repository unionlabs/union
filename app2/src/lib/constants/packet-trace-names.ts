export type DisplayMode = "packet" | "transfer"

export const PACKET_TRACE_DISPLAY_NAMES: Record<DisplayMode, Record<string, string>> = {
  packet: {
    PACKET_SEND: "Packet sent",
    PACKET_SEND_LC_UPDATE_L2: "Light Client L2 updated",
    PACKET_SEND_LC_UPDATE_L1: "Light Client L1 updated",
    PACKET_SEND_LC_UPDATE_L0: "Light Client L0 updated",
    PACKET_RECV: "Packet received",
    WRITE_ACK: "Acknowledgement written",
    WRITE_ACK_LC_UPDATE_L0: "Light Client L0 updated for ack",
    WRITE_ACK_LC_UPDATE_L1: "Light Client L1 updated for ack",
    WRITE_ACK_LC_UPDATE_L2: "Light Client L2 updated for ack",
    PACKET_ACK: "Packet acknowledged"
  },
  transfer: {
    PACKET_SEND: "Transfer sent",
    PACKET_SEND_LC_UPDATE_L2: "Light Client L2 updated",
    PACKET_SEND_LC_UPDATE_L1: "Light Client L1 updated",
    PACKET_SEND_LC_UPDATE_L0: "Light Client L0 updated",
    PACKET_RECV: "Transfer received",
    WRITE_ACK: "Acknowledgement written",
    WRITE_ACK_LC_UPDATE_L0: "Light Client L0 updated for ack",
    WRITE_ACK_LC_UPDATE_L1: "Light Client L1 updated for ack",
    WRITE_ACK_LC_UPDATE_L2: "Light Client L2 updated for ack",
    PACKET_ACK: "Transfer acknowledged"
  }
}

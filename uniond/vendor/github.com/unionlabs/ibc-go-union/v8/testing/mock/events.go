package mock

import sdk "github.com/cosmos/cosmos-sdk/types"

const (
	MockEventType              = "mock-event-type"
	MockEventTypeRecvPacket    = "mock-recv-packet"
	MockEventTypeAckPacket     = "mock-ack-packet"
	MockEventTypeTimeoutPacket = "mock-timeout"

	MockAttributeKey1 = "mock-attribute-key-1"
	MockAttributeKey2 = "mock-attribute-key-2"

	MockAttributeValue1 = "mock-attribute-value-1"
	MockAttributeValue2 = "mock-attribute-value-2"
)

// NewMockRecvPacketEvent returns a mock receive packet event
func NewMockRecvPacketEvent() sdk.Event {
	return newMockEvent(MockEventTypeRecvPacket)
}

// NewMockAckPacketEvent returns a mock acknowledgement packet event
func NewMockAckPacketEvent() sdk.Event {
	return newMockEvent(MockEventTypeAckPacket)
}

// NewMockTimeoutPacketEvent emits a mock timeout packet event
func NewMockTimeoutPacketEvent() sdk.Event {
	return newMockEvent(MockEventTypeTimeoutPacket)
}

// emitMockEvent returns a mock event with the given event type
func newMockEvent(eventType string) sdk.Event {
	return sdk.NewEvent(
		eventType,
		sdk.NewAttribute(MockAttributeKey1, MockAttributeValue1),
		sdk.NewAttribute(MockAttributeKey2, MockAttributeValue2),
	)
}

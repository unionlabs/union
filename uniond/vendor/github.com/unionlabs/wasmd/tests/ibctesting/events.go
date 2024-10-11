package ibctesting

import (
	"encoding/hex"
	"fmt"
	"strconv"

	abci "github.com/cometbft/cometbft/abci/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	"github.com/stretchr/testify/suite"
)

type EventsMap map[string]map[string]string

// ParseClientIDFromEvents parses events emitted from a MsgCreateClient and returns the
// client identifier.
func ParseClientIDFromEvents(events []abci.Event) (string, error) {
	for _, ev := range events {
		if ev.Type == clienttypes.EventTypeCreateClient {
			for _, attr := range ev.Attributes {
				if attr.Key == clienttypes.AttributeKeyClientID {
					return attr.Value, nil
				}
			}
		}
	}
	return "", fmt.Errorf("client identifier event attribute not found")
}

// ParseConnectionIDFromEvents parses events emitted from a MsgConnectionOpenInit or
// MsgConnectionOpenTry and returns the connection identifier.
func ParseConnectionIDFromEvents(events []abci.Event) (string, error) {
	for _, ev := range events {
		if ev.Type == connectiontypes.EventTypeConnectionOpenInit ||
			ev.Type == connectiontypes.EventTypeConnectionOpenTry {
			for _, attr := range ev.Attributes {
				if attr.Key == connectiontypes.AttributeKeyConnectionID {
					return attr.Value, nil
				}
			}
		}
	}
	return "", fmt.Errorf("connection identifier event attribute not found")
}

// ParseChannelIDFromEvents parses events emitted from a MsgChannelOpenInit or
// MsgChannelOpenTry and returns the channel identifier.
func ParseChannelIDFromEvents(events []abci.Event) (string, error) {
	for _, ev := range events {
		if ev.Type == channeltypes.EventTypeChannelOpenInit || ev.Type == channeltypes.EventTypeChannelOpenTry {
			for _, attr := range ev.Attributes {
				if attr.Key == channeltypes.AttributeKeyChannelID {
					return attr.Value, nil
				}
			}
		}
	}
	return "", fmt.Errorf("channel identifier event attribute not found")
}

// ParsePacketFromEvents parses events emitted from a MsgRecvPacket and returns the
// acknowledgement.
func ParsePacketFromEvents(events []abci.Event) (channeltypes.Packet, error) {
	for _, ev := range events {
		if ev.Type == channeltypes.EventTypeSendPacket {
			packet := channeltypes.Packet{}
			for _, attr := range ev.Attributes {
				switch attr.Key {
				case channeltypes.AttributeKeyDataHex:
					bz, err := hex.DecodeString(attr.Value)
					if err != nil {
						panic(err)
					}
					packet.Data = bz

				case channeltypes.AttributeKeySequence:
					seq, err := strconv.ParseUint(attr.Value, 10, 64)
					if err != nil {
						return channeltypes.Packet{}, err
					}

					packet.Sequence = seq

				case channeltypes.AttributeKeySrcPort:
					packet.SourcePort = attr.Value

				case channeltypes.AttributeKeySrcChannel:
					packet.SourceChannel = attr.Value

				case channeltypes.AttributeKeyDstPort:
					packet.DestinationPort = attr.Value

				case channeltypes.AttributeKeyDstChannel:
					packet.DestinationChannel = attr.Value

				case channeltypes.AttributeKeyTimeoutHeight:
					height, err := clienttypes.ParseHeight(attr.Value)
					if err != nil {
						return channeltypes.Packet{}, err
					}

					packet.TimeoutHeight = height

				case channeltypes.AttributeKeyTimeoutTimestamp:
					timestamp, err := strconv.ParseUint(attr.Value, 10, 64)
					if err != nil {
						return channeltypes.Packet{}, err
					}

					packet.TimeoutTimestamp = timestamp

				default:
					continue
				}
			}

			return packet, nil
		}
	}
	return channeltypes.Packet{}, fmt.Errorf("acknowledgement event attribute not found")
}

// ParseAckFromEvents parses events emitted from a MsgRecvPacket and returns the
// acknowledgement.
func ParseAckFromEvents(events []abci.Event) ([]byte, error) {
	for _, ev := range events {
		if ev.Type == channeltypes.EventTypeWriteAck {
			for _, attr := range ev.Attributes {
				if attr.Key == channeltypes.AttributeKeyAckHex {
					bz, err := hex.DecodeString(attr.Value)
					if err != nil {
						panic(err)
					}
					return bz, nil
				}
			}
		}
	}
	return nil, fmt.Errorf("acknowledgement event attribute not found")
}

// AssertEvents asserts that expected events are present in the actual events.
// Expected map needs to be a subset of actual events to pass.
func AssertEvents(
	suite *suite.Suite,
	expected EventsMap,
	actual []abci.Event,
) {
	hasEvents := make(map[string]bool)
	for eventType := range expected {
		hasEvents[eventType] = false
	}

	for _, event := range actual {
		expEvent, eventFound := expected[event.Type]
		if eventFound {
			hasEvents[event.Type] = true
			suite.Require().Len(event.Attributes, len(expEvent))
			for _, attr := range event.Attributes {
				expValue, found := expEvent[attr.Key]
				suite.Require().True(found)
				suite.Require().Equal(expValue, attr.Value)
			}
		}
	}

	for eventName, hasEvent := range hasEvents {
		suite.Require().True(hasEvent, "event: %s was not found in events", eventName)
	}
}

package p2p

import (
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/math"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
)

// MsgID is a content addressable ID function.
//
// Ethereum Beacon Chain spec defines the message ID as:
//
//	The `message-id` of a gossipsub message MUST be the following 20 byte value computed from the message data:
//	If `message.data` has a valid snappy decompression, set `message-id` to the first 20 bytes of the `SHA256` hash of
//	the concatenation of `MESSAGE_DOMAIN_VALID_SNAPPY` with the snappy decompressed message data,
//	i.e. `SHA256(MESSAGE_DOMAIN_VALID_SNAPPY + snappy_decompress(message.data))[:20]`.
//
//	Otherwise, set `message-id` to the first 20 bytes of the `SHA256` hash of
//	the concatenation of `MESSAGE_DOMAIN_INVALID_SNAPPY` with the raw message data,
//	i.e. `SHA256(MESSAGE_DOMAIN_INVALID_SNAPPY + message.data)[:20]`.
func MsgID(genesisValidatorsRoot []byte, pmsg *pubsubpb.Message) string {
	if pmsg == nil || pmsg.Data == nil || pmsg.Topic == nil {
		// Impossible condition that should
		// never be hit.
		msg := make([]byte, 20)
		copy(msg, "invalid")
		return string(msg)
	}
	digest, err := ExtractGossipDigest(*pmsg.Topic)
	if err != nil {
		// Impossible condition that should
		// never be hit.
		msg := make([]byte, 20)
		copy(msg, "invalid")
		return string(msg)
	}
	_, fEpoch, err := forks.RetrieveForkDataFromDigest(digest, genesisValidatorsRoot)
	if err != nil {
		// Impossible condition that should
		// never be hit.
		msg := make([]byte, 20)
		copy(msg, "invalid")
		return string(msg)
	}
	if fEpoch >= params.BeaconConfig().AltairForkEpoch {
		return postAltairMsgID(pmsg, fEpoch)
	}
	decodedData, err := encoder.DecodeSnappy(pmsg.Data, params.BeaconConfig().GossipMaxSize)
	if err != nil {
		combinedData := append(params.BeaconConfig().MessageDomainInvalidSnappy[:], pmsg.Data...)
		h := hash.Hash(combinedData)
		return string(h[:20])
	}
	combinedData := append(params.BeaconConfig().MessageDomainValidSnappy[:], decodedData...)
	h := hash.Hash(combinedData)
	return string(h[:20])
}

// Spec:
// The derivation of the message-id has changed starting with Altair to incorporate the message topic along with the message data.
// These are fields of the Message Protobuf, and interpreted as empty byte strings if missing. The message-id MUST be the following
// 20 byte value computed from the message:
//
// If message.data has a valid snappy decompression, set message-id to the first 20 bytes of the SHA256 hash of the concatenation of
// the following data: MESSAGE_DOMAIN_VALID_SNAPPY, the length of the topic byte string (encoded as little-endian uint64), the topic
// byte string, and the snappy decompressed message data: i.e. SHA256(MESSAGE_DOMAIN_VALID_SNAPPY + uint_to_bytes(uint64(len(message.topic)))
// + message.topic + snappy_decompress(message.data))[:20]. Otherwise, set message-id to the first 20 bytes of the SHA256 hash of the concatenation
// of the following data: MESSAGE_DOMAIN_INVALID_SNAPPY, the length of the topic byte string (encoded as little-endian uint64),
// the topic byte string, and the raw message data: i.e. SHA256(MESSAGE_DOMAIN_INVALID_SNAPPY + uint_to_bytes(uint64(len(message.topic))) + message.topic + message.data)[:20].
func postAltairMsgID(pmsg *pubsubpb.Message, fEpoch primitives.Epoch) string {
	topic := *pmsg.Topic
	topicLen := len(topic)
	topicLenBytes := bytesutil.Uint64ToBytesLittleEndian(uint64(topicLen)) // topicLen cannot be negative

	// beyond Bellatrix epoch, allow 10 Mib gossip data size
	gossipPubSubSize := params.BeaconConfig().GossipMaxSize

	decodedData, err := encoder.DecodeSnappy(pmsg.Data, gossipPubSubSize)
	if err != nil {
		totalLength, err := math.AddInt(
			len(params.BeaconConfig().MessageDomainInvalidSnappy),
			len(topicLenBytes),
			topicLen,
			len(pmsg.Data),
		)
		if err != nil {
			log.WithError(err).Error("Failed to sum lengths of message domain and topic")
			// should never happen
			msg := make([]byte, 20)
			copy(msg, "invalid")
			return string(msg)
		}
		if uint64(totalLength) > gossipPubSubSize {
			// this should never happen
			msg := make([]byte, 20)
			copy(msg, "invalid")
			return string(msg)
		}
		combinedData := make([]byte, 0, totalLength)
		combinedData = append(combinedData, params.BeaconConfig().MessageDomainInvalidSnappy[:]...)
		combinedData = append(combinedData, topicLenBytes...)
		combinedData = append(combinedData, topic...)
		combinedData = append(combinedData, pmsg.Data...)
		h := hash.Hash(combinedData)
		return string(h[:20])
	}
	totalLength, err := math.AddInt(
		len(params.BeaconConfig().MessageDomainValidSnappy),
		len(topicLenBytes),
		topicLen,
		len(decodedData),
	)
	if err != nil {
		log.WithError(err).Error("Failed to sum lengths of message domain and topic")
		// should never happen
		msg := make([]byte, 20)
		copy(msg, "invalid")
		return string(msg)
	}
	combinedData := make([]byte, 0, totalLength)
	combinedData = append(combinedData, params.BeaconConfig().MessageDomainValidSnappy[:]...)
	combinedData = append(combinedData, topicLenBytes...)
	combinedData = append(combinedData, topic...)
	combinedData = append(combinedData, decodedData...)
	h := hash.Hash(combinedData)
	return string(h[:20])
}

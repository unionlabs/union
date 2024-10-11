package p2p_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/golang/snappy"
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestMsgID_HashesCorrectly(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	genesisValidatorsRoot := bytesutil.PadTo([]byte{'A'}, 32)
	d, err := forks.CreateForkDigest(time.Now(), genesisValidatorsRoot)
	assert.NoError(t, err)
	tpc := fmt.Sprintf(p2p.BlockSubnetTopicFormat, d)
	invalidSnappy := [32]byte{'J', 'U', 'N', 'K'}
	pMsg := &pubsubpb.Message{Data: invalidSnappy[:], Topic: &tpc}
	hashedData := hash.Hash(append(params.BeaconConfig().MessageDomainInvalidSnappy[:], pMsg.Data...))
	msgID := string(hashedData[:20])
	assert.Equal(t, msgID, p2p.MsgID(genesisValidatorsRoot, pMsg), "Got incorrect msg id")

	validObj := [32]byte{'v', 'a', 'l', 'i', 'd'}
	enc := snappy.Encode(nil, validObj[:])
	nMsg := &pubsubpb.Message{Data: enc, Topic: &tpc}
	hashedData = hash.Hash(append(params.BeaconConfig().MessageDomainValidSnappy[:], validObj[:]...))
	msgID = string(hashedData[:20])
	assert.Equal(t, msgID, p2p.MsgID(genesisValidatorsRoot, nMsg), "Got incorrect msg id")
}

func TestMessageIDFunction_HashesCorrectlyAltair(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	genesisValidatorsRoot := bytesutil.PadTo([]byte{'A'}, 32)
	d, err := signing.ComputeForkDigest(params.BeaconConfig().AltairForkVersion, genesisValidatorsRoot)
	assert.NoError(t, err)
	tpc := fmt.Sprintf(p2p.BlockSubnetTopicFormat, d)
	topicLen := uint64(len(tpc))
	topicLenBytes := bytesutil.Uint64ToBytesLittleEndian(topicLen)
	invalidSnappy := [32]byte{'J', 'U', 'N', 'K'}
	pMsg := &pubsubpb.Message{Data: invalidSnappy[:], Topic: &tpc}
	// Create object to hash
	combinedObj := append(params.BeaconConfig().MessageDomainInvalidSnappy[:], topicLenBytes...)
	combinedObj = append(combinedObj, tpc...)
	combinedObj = append(combinedObj, pMsg.Data...)
	hashedData := hash.Hash(combinedObj)
	msgID := string(hashedData[:20])
	assert.Equal(t, msgID, p2p.MsgID(genesisValidatorsRoot, pMsg), "Got incorrect msg id")

	validObj := [32]byte{'v', 'a', 'l', 'i', 'd'}
	enc := snappy.Encode(nil, validObj[:])
	nMsg := &pubsubpb.Message{Data: enc, Topic: &tpc}
	// Create object to hash
	combinedObj = append(params.BeaconConfig().MessageDomainValidSnappy[:], topicLenBytes...)
	combinedObj = append(combinedObj, tpc...)
	combinedObj = append(combinedObj, validObj[:]...)
	hashedData = hash.Hash(combinedObj)
	msgID = string(hashedData[:20])
	assert.Equal(t, msgID, p2p.MsgID(genesisValidatorsRoot, nMsg), "Got incorrect msg id")
}

func TestMessageIDFunction_HashesCorrectlyBellatrix(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	genesisValidatorsRoot := bytesutil.PadTo([]byte{'A'}, 32)
	d, err := signing.ComputeForkDigest(params.BeaconConfig().BellatrixForkVersion, genesisValidatorsRoot)
	assert.NoError(t, err)
	tpc := fmt.Sprintf(p2p.BlockSubnetTopicFormat, d)
	topicLen := uint64(len(tpc))
	topicLenBytes := bytesutil.Uint64ToBytesLittleEndian(topicLen)
	invalidSnappy := [32]byte{'J', 'U', 'N', 'K'}
	pMsg := &pubsubpb.Message{Data: invalidSnappy[:], Topic: &tpc}
	// Create object to hash
	combinedObj := append(params.BeaconConfig().MessageDomainInvalidSnappy[:], topicLenBytes...)
	combinedObj = append(combinedObj, tpc...)
	combinedObj = append(combinedObj, pMsg.Data...)
	hashedData := hash.Hash(combinedObj)
	msgID := string(hashedData[:20])
	assert.Equal(t, msgID, p2p.MsgID(genesisValidatorsRoot, pMsg), "Got incorrect msg id")

	validObj := [32]byte{'v', 'a', 'l', 'i', 'd'}
	enc := snappy.Encode(nil, validObj[:])
	nMsg := &pubsubpb.Message{Data: enc, Topic: &tpc}
	// Create object to hash
	combinedObj = append(params.BeaconConfig().MessageDomainValidSnappy[:], topicLenBytes...)
	combinedObj = append(combinedObj, tpc...)
	combinedObj = append(combinedObj, validObj[:]...)
	hashedData = hash.Hash(combinedObj)
	msgID = string(hashedData[:20])
	assert.Equal(t, msgID, p2p.MsgID(genesisValidatorsRoot, nMsg), "Got incorrect msg id")
}

func TestMsgID_WithNilTopic(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	msg := &pubsubpb.Message{
		Data:  make([]byte, 32),
		Topic: nil,
	}

	invalid := make([]byte, 20)
	copy(invalid, "invalid")

	res := p2p.MsgID([]byte{0x01}, msg)
	assert.Equal(t, res, string(invalid))
}

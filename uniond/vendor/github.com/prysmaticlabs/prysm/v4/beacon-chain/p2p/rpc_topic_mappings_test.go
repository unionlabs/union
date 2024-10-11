package p2p

import (
	"fmt"
	"strings"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestVerifyRPCMappings(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	assert.NoError(t, VerifyTopicMapping(RPCStatusTopicV1, &pb.Status{}), "Failed to verify status rpc topic")
	assert.NotNil(t, VerifyTopicMapping(RPCStatusTopicV1, new([]byte)), "Incorrect message type verified for status rpc topic")

	assert.NoError(t, VerifyTopicMapping(RPCMetaDataTopicV1, new(interface{})), "Failed to verify metadata rpc topic")
	assert.NotNil(t, VerifyTopicMapping(RPCStatusTopicV1, new([]byte)), "Incorrect message type verified for metadata rpc topic")

	assert.NoError(t, VerifyTopicMapping(RPCBlocksByRootTopicV1, new(types.BeaconBlockByRootsReq)), "Failed to verify blocks by root rpc topic")
}

func TestTopicDeconstructor(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	tt := []struct {
		name          string
		topic         string
		expectedError string
		output        []string
	}{
		{
			name:          "invalid topic",
			topic:         "/sjdksfks/dusidsdsd/ssz",
			expectedError: "unable to find a valid protocol prefix for /sjdksfks/dusidsdsd/ssz",
			output:        []string{"", "", ""},
		},
		{
			name:          "valid status topic",
			topic:         protocolPrefix + StatusMessageName + SchemaVersionV1,
			expectedError: "",
			output:        []string{protocolPrefix, StatusMessageName, SchemaVersionV1},
		},
		{
			name:          "malformed status topic",
			topic:         protocolPrefix + "/statis" + SchemaVersionV1,
			expectedError: "unable to find a valid message for /eth2/beacon_chain/req/statis/1",
			output:        []string{""},
		},
		{
			name:          "valid beacon block by range topic",
			topic:         protocolPrefix + BeaconBlocksByRangeMessageName + SchemaVersionV1 + "/ssz_snappy",
			expectedError: "",
			output:        []string{protocolPrefix, BeaconBlocksByRangeMessageName, SchemaVersionV1},
		},
		{
			name:          "beacon block by range topic with malformed version",
			topic:         protocolPrefix + BeaconBlocksByRangeMessageName + "/v" + "/ssz_snappy",
			expectedError: "unable to find a valid schema version for /eth2/beacon_chain/req/beacon_blocks_by_range/v/ssz_snappy",
			output:        []string{""},
		},
	}

	for _, test := range tt {
		t.Run(test.name, func(t *testing.T) {
			protocolPref, message, version, err := TopicDeconstructor(test.topic)
			if test.expectedError != "" {
				require.NotNil(t, err)
				assert.Equal(t, test.expectedError, err.Error())
			} else {
				require.NoError(t, err)
				assert.Equal(t, test.output[0], protocolPref)
				assert.Equal(t, test.output[1], message)
				assert.Equal(t, test.output[2], version)
			}
		})
	}
}

func TestTopicFromMessage_CorrectType(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bCfg := params.BeaconConfig().Copy()
	forkEpoch := primitives.Epoch(100)
	bCfg.AltairForkEpoch = forkEpoch
	bCfg.ForkVersionSchedule[bytesutil.ToBytes4(bCfg.AltairForkVersion)] = primitives.Epoch(100)
	params.OverrideBeaconConfig(bCfg)

	// Garbage Message
	badMsg := "wljdjska"
	_, err := TopicFromMessage(badMsg, 0)
	assert.ErrorContains(t, fmt.Sprintf("%s: %s", invalidRPCMessageType, badMsg), err)
	// Before Fork
	for m := range messageMapping {
		topic, err := TopicFromMessage(m, 0)
		assert.NoError(t, err)

		assert.Equal(t, true, strings.Contains(topic, SchemaVersionV1))
		_, _, version, err := TopicDeconstructor(topic)
		assert.NoError(t, err)
		assert.Equal(t, SchemaVersionV1, version)
	}

	// Altair Fork
	for m := range messageMapping {
		topic, err := TopicFromMessage(m, forkEpoch)
		assert.NoError(t, err)

		if altairMapping[m] {
			assert.Equal(t, true, strings.Contains(topic, SchemaVersionV2))
			_, _, version, err := TopicDeconstructor(topic)
			assert.NoError(t, err)
			assert.Equal(t, SchemaVersionV2, version)
		} else {
			assert.Equal(t, true, strings.Contains(topic, SchemaVersionV1))
			_, _, version, err := TopicDeconstructor(topic)
			assert.NoError(t, err)
			assert.Equal(t, SchemaVersionV1, version)
		}
	}
}

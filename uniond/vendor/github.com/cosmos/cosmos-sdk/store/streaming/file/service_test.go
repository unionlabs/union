package file

import (
	"encoding/binary"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codecTypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	interfaceRegistry            = codecTypes.NewInterfaceRegistry()
	testMarshaller               = codec.NewProtoCodec(interfaceRegistry)
	testStreamingService         *StreamingService
	testListener1, testListener2 types.WriteListener
	emptyContext                 = sdk.Context{}

	// test abci message types
	mockHash          = []byte{1, 2, 3, 4, 5, 6, 7, 8, 9}
	testBeginBlockReq = abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height: 1,
		},
		ByzantineValidators: []abci.Misbehavior{},
		Hash:                mockHash,
		LastCommitInfo: abci.CommitInfo{
			Round: 1,
			Votes: []abci.VoteInfo{},
		},
	}
	testBeginBlockRes = abci.ResponseBeginBlock{
		Events: []abci.Event{
			{
				Type: "testEventType1",
			},
			{
				Type: "testEventType2",
			},
		},
	}
	testEndBlockReq = abci.RequestEndBlock{
		Height: 1,
	}
	testEndBlockRes = abci.ResponseEndBlock{
		Events:                []abci.Event{},
		ConsensusParamUpdates: &tmproto.ConsensusParams{},
		ValidatorUpdates:      []abci.ValidatorUpdate{},
	}
	testCommitRes = abci.ResponseCommit{
		Data:         []byte{1},
		RetainHeight: 0,
	}
	mockTxBytes1      = []byte{9, 8, 7, 6, 5, 4, 3, 2, 1}
	testDeliverTxReq1 = abci.RequestDeliverTx{
		Tx: mockTxBytes1,
	}
	mockTxBytes2      = []byte{8, 7, 6, 5, 4, 3, 2}
	testDeliverTxReq2 = abci.RequestDeliverTx{
		Tx: mockTxBytes2,
	}
	mockTxResponseData1 = []byte{1, 3, 5, 7, 9}
	testDeliverTxRes1   = abci.ResponseDeliverTx{
		Events:    []abci.Event{},
		Code:      1,
		Codespace: "mockCodeSpace",
		Data:      mockTxResponseData1,
		GasUsed:   2,
		GasWanted: 3,
		Info:      "mockInfo",
		Log:       "mockLog",
	}
	mockTxResponseData2 = []byte{1, 3, 5, 7, 9}
	testDeliverTxRes2   = abci.ResponseDeliverTx{
		Events:    []abci.Event{},
		Code:      1,
		Codespace: "mockCodeSpace",
		Data:      mockTxResponseData2,
		GasUsed:   2,
		GasWanted: 3,
		Info:      "mockInfo",
		Log:       "mockLog",
	}

	// mock store keys
	mockStoreKey1 = sdk.NewKVStoreKey("mockStore1")
	mockStoreKey2 = sdk.NewKVStoreKey("mockStore2")

	// file stuff
	testPrefix = "testPrefix"
	testDir    = "./.test"

	// mock state changes
	mockKey1   = []byte{1, 2, 3}
	mockValue1 = []byte{3, 2, 1}
	mockKey2   = []byte{2, 3, 4}
	mockValue2 = []byte{4, 3, 2}
	mockKey3   = []byte{3, 4, 5}
	mockValue3 = []byte{5, 4, 3}
)

func TestFileStreamingService(t *testing.T) {
	if os.Getenv("CI") != "" {
		t.Skip("Skipping TestFileStreamingService in CI environment")
	}

	require.Nil(t, os.Mkdir(testDir, 0o700))
	defer os.RemoveAll(testDir)

	testKeys := []types.StoreKey{mockStoreKey1, mockStoreKey2}
	var err error
	testStreamingService, err = NewStreamingService(testDir, testPrefix, testKeys, testMarshaller, log.NewNopLogger(), true, false, false)
	require.Nil(t, err)
	require.IsType(t, &StreamingService{}, testStreamingService)
	require.Equal(t, testPrefix, testStreamingService.filePrefix)
	require.Equal(t, testDir, testStreamingService.writeDir)
	require.Equal(t, testMarshaller, testStreamingService.codec)

	testListener1 = testStreamingService.storeListeners[0]
	testListener2 = testStreamingService.storeListeners[1]

	wg := new(sync.WaitGroup)

	testStreamingService.Stream(wg)
	testListenBlock(t)
	testStreamingService.Close()
	wg.Wait()
}

func testListenBlock(t *testing.T) {
	var (
		expectKVPairsStore1 [][]byte
		expectKVPairsStore2 [][]byte
	)

	// write state changes
	testListener1.OnWrite(mockStoreKey1, mockKey1, mockValue1, false)
	testListener2.OnWrite(mockStoreKey2, mockKey2, mockValue2, false)
	testListener1.OnWrite(mockStoreKey1, mockKey3, mockValue3, false)

	// expected KV pairs
	expectedKVPair1, err := testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey1.Name(),
		Key:      mockKey1,
		Value:    mockValue1,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair2, err := testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey2.Name(),
		Key:      mockKey2,
		Value:    mockValue2,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair3, err := testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey1.Name(),
		Key:      mockKey3,
		Value:    mockValue3,
		Delete:   false,
	})
	require.Nil(t, err)

	expectKVPairsStore1 = append(expectKVPairsStore1, expectedKVPair1, expectedKVPair3)
	expectKVPairsStore2 = append(expectKVPairsStore2, expectedKVPair2)

	// send the ABCI messages
	err = testStreamingService.ListenBeginBlock(emptyContext, testBeginBlockReq, testBeginBlockRes)
	require.Nil(t, err)

	// write state changes
	testListener1.OnWrite(mockStoreKey1, mockKey1, mockValue1, false)
	testListener2.OnWrite(mockStoreKey2, mockKey2, mockValue2, false)
	testListener2.OnWrite(mockStoreKey2, mockKey3, mockValue3, false)

	// expected KV pairs
	expectedKVPair1, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey1.Name(),
		Key:      mockKey1,
		Value:    mockValue1,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair2, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey2.Name(),
		Key:      mockKey2,
		Value:    mockValue2,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair3, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey2.Name(),
		Key:      mockKey3,
		Value:    mockValue3,
		Delete:   false,
	})
	require.Nil(t, err)

	expectKVPairsStore1 = append(expectKVPairsStore1, expectedKVPair1)
	expectKVPairsStore2 = append(expectKVPairsStore2, expectedKVPair2, expectedKVPair3)

	// send the ABCI messages
	err = testStreamingService.ListenDeliverTx(emptyContext, testDeliverTxReq1, testDeliverTxRes1)
	require.Nil(t, err)

	// write state changes
	testListener2.OnWrite(mockStoreKey2, mockKey1, mockValue1, false)
	testListener1.OnWrite(mockStoreKey1, mockKey2, mockValue2, false)
	testListener2.OnWrite(mockStoreKey2, mockKey3, mockValue3, false)

	// expected KV pairs
	expectedKVPair1, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey2.Name(),
		Key:      mockKey1,
		Value:    mockValue1,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair2, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey1.Name(),
		Key:      mockKey2,
		Value:    mockValue2,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair3, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey2.Name(),
		Key:      mockKey3,
		Value:    mockValue3,
		Delete:   false,
	})
	require.Nil(t, err)

	expectKVPairsStore1 = append(expectKVPairsStore1, expectedKVPair2)
	expectKVPairsStore2 = append(expectKVPairsStore2, expectedKVPair1, expectedKVPair3)

	// send the ABCI messages
	err = testStreamingService.ListenDeliverTx(emptyContext, testDeliverTxReq2, testDeliverTxRes2)
	require.Nil(t, err)

	// write state changes
	testListener1.OnWrite(mockStoreKey1, mockKey1, mockValue1, false)
	testListener1.OnWrite(mockStoreKey1, mockKey2, mockValue2, false)
	testListener2.OnWrite(mockStoreKey2, mockKey3, mockValue3, false)

	// expected KV pairs
	expectedKVPair1, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey1.Name(),
		Key:      mockKey1,
		Value:    mockValue1,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair2, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey1.Name(),
		Key:      mockKey2,
		Value:    mockValue2,
		Delete:   false,
	})
	require.Nil(t, err)

	expectedKVPair3, err = testMarshaller.Marshal(&types.StoreKVPair{
		StoreKey: mockStoreKey2.Name(),
		Key:      mockKey3,
		Value:    mockValue3,
		Delete:   false,
	})
	require.Nil(t, err)

	expectKVPairsStore1 = append(expectKVPairsStore1, expectedKVPair1, expectedKVPair2)
	expectKVPairsStore2 = append(expectKVPairsStore2, expectedKVPair3)

	// send the ABCI messages
	err = testStreamingService.ListenEndBlock(emptyContext, testEndBlockReq, testEndBlockRes)
	require.Nil(t, err)

	err = testStreamingService.ListenCommit(emptyContext, testCommitRes)
	require.Nil(t, err)

	// load the file, checking that it was created with the expected name
	metaFileName := fmt.Sprintf("%s-block-%d-meta", testPrefix, testBeginBlockReq.GetHeader().Height)
	dataFileName := fmt.Sprintf("%s-block-%d-data", testPrefix, testBeginBlockReq.GetHeader().Height)
	metaFileBytes, err := readInFile(metaFileName)
	require.Nil(t, err)
	dataFileBytes, err := readInFile(dataFileName)
	require.Nil(t, err)

	metadata := types.BlockMetadata{
		RequestBeginBlock:  &testBeginBlockReq,
		ResponseBeginBlock: &testBeginBlockRes,
		RequestEndBlock:    &testEndBlockReq,
		ResponseEndBlock:   &testEndBlockRes,
		ResponseCommit:     &testCommitRes,
		DeliverTxs: []*types.BlockMetadata_DeliverTx{
			{Request: &testDeliverTxReq1, Response: &testDeliverTxRes1},
			{Request: &testDeliverTxReq2, Response: &testDeliverTxRes2},
		},
	}
	expectedMetadataBytes, err := testMarshaller.Marshal(&metadata)
	require.Nil(t, err)
	require.Equal(t, expectedMetadataBytes, metaFileBytes)

	// segment the file into the separate gRPC messages and check the correctness of each
	segments, err := segmentBytes(dataFileBytes)
	require.Nil(t, err)
	require.Equal(t, len(expectKVPairsStore1)+len(expectKVPairsStore2), len(segments))
	require.Equal(t, expectKVPairsStore1, segments[:len(expectKVPairsStore1)])
	require.Equal(t, expectKVPairsStore2, segments[len(expectKVPairsStore1):])
}

func readInFile(name string) ([]byte, error) {
	path := filepath.Join(testDir, name)
	bz, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	size := sdk.BigEndianToUint64(bz[:8])
	if len(bz) != int(size)+8 {
		return nil, errors.New("incomplete file ")
	}

	return bz[8:], nil
}

// segmentBytes returns all of the protobuf messages contained in the byte array
// as an array of byte arrays. The messages have their length prefix removed.
func segmentBytes(bz []byte) ([][]byte, error) {
	var err error

	segments := make([][]byte, 0)
	for len(bz) > 0 {
		var segment []byte

		segment, bz, err = getHeadSegment(bz)
		if err != nil {
			return nil, err
		}

		segments = append(segments, segment)
	}

	return segments, nil
}

// getHeadSegment returns the bytes for the leading protobuf object in the byte
// array (removing the length prefix) and returns the remainder of the byte array.
func getHeadSegment(bz []byte) ([]byte, []byte, error) {
	size, prefixSize := binary.Uvarint(bz)
	if prefixSize < 0 {
		return nil, nil, fmt.Errorf("invalid number of bytes read from length-prefixed encoding: %d", prefixSize)
	}

	if size > uint64(len(bz)-prefixSize) {
		return nil, nil, fmt.Errorf("not enough bytes to read; want: %v, got: %v", size, len(bz)-prefixSize)
	}

	return bz[prefixSize:(uint64(prefixSize) + size)], bz[uint64(prefixSize)+size:], nil
}

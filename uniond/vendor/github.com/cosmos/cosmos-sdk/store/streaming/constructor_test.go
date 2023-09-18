package streaming_test

import (
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	codecTypes "github.com/cosmos/cosmos-sdk/codec/types"
	serverTypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/store/streaming"
	"github.com/cosmos/cosmos-sdk/store/streaming/file"
	"github.com/cosmos/cosmos-sdk/store/types"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
)

type fakeOptions struct{}

func (f *fakeOptions) Get(key string) interface{} {
	if key == "streamers.file.write_dir" {
		return "data/file_streamer"
	}
	return nil
}

var (
	mockOptions       = new(fakeOptions)
	mockKeys          = []types.StoreKey{sdk.NewKVStoreKey("mockKey1"), sdk.NewKVStoreKey("mockKey2")}
	interfaceRegistry = codecTypes.NewInterfaceRegistry()
	testMarshaller    = codec.NewProtoCodec(interfaceRegistry)
)

func TestStreamingServiceConstructor(t *testing.T) {
	_, err := streaming.NewServiceConstructor("unexpectedName")
	require.NotNil(t, err)

	constructor, err := streaming.NewServiceConstructor("file")
	require.Nil(t, err)
	var expectedType streaming.ServiceConstructor
	require.IsType(t, expectedType, constructor)

	serv, err := constructor(mockOptions, mockKeys, testMarshaller, log.NewNopLogger())
	require.Nil(t, err)
	require.IsType(t, &file.StreamingService{}, serv)
	listeners := serv.Listeners()
	for _, key := range mockKeys {
		_, ok := listeners[key]
		require.True(t, ok)
	}
}

func TestLoadStreamingServices(t *testing.T) {
	db := dbm.NewMemDB()
	encCdc := testutil.MakeTestEncodingConfig()
	keys := sdk.NewKVStoreKeys("mockKey1", "mockKey2")
	bApp := baseapp.NewBaseApp("appName", log.NewNopLogger(), db, nil)

	testCases := map[string]struct {
		appOpts            serverTypes.AppOptions
		activeStreamersLen int
	}{
		"empty app options": {
			appOpts: simtestutil.EmptyAppOptions{},
		},
		"all StoreKeys exposed": {
			appOpts:            streamingAppOptions{keys: []string{"*"}},
			activeStreamersLen: 1,
		},
		"some StoreKey exposed": {
			appOpts:            streamingAppOptions{keys: []string{"mockKey1"}},
			activeStreamersLen: 1,
		},
		"not exposing anything": {
			appOpts: streamingAppOptions{keys: []string{"mockKey3"}},
		},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			activeStreamers, _, err := streaming.LoadStreamingServices(bApp, tc.appOpts, encCdc.Codec, log.NewNopLogger(), keys)
			require.NoError(t, err)
			require.Equal(t, tc.activeStreamersLen, len(activeStreamers))
		})
	}
}

type streamingAppOptions struct {
	keys []string
}

func (ao streamingAppOptions) Get(o string) interface{} {
	switch o {
	case "store.streamers":
		return []string{"file"}
	case "streamers.file.keys":
		return ao.keys
	case "streamers.file.write_dir":
		return "data/file_streamer"
	default:
		return nil
	}
}

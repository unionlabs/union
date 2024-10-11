package indexer

import (
	"context"
	"encoding/json"
	"fmt"
	"reflect"
	"sync"

	"cosmossdk.io/schema/addressutil"
	"cosmossdk.io/schema/appdata"
	"cosmossdk.io/schema/decoding"
	"cosmossdk.io/schema/logutil"
	"cosmossdk.io/schema/view"
)

// IndexingOptions are the options for starting the indexer manager.
type IndexingOptions struct {
	// Config is the user configuration for all indexing. It should generally be an instance map[string]interface{}
	// or json.RawMessage and match the json structure of IndexingConfig, or it can be an instance of IndexingConfig.
	// The manager will attempt to convert it to IndexingConfig.
	Config interface{}

	// Resolver is the decoder resolver that will be used to decode the data. It is required.
	Resolver decoding.DecoderResolver

	// SyncSource is a representation of the current state of key-value data to be used in a catch-up sync.
	// Catch-up syncs will be performed at initialization when necessary. SyncSource is optional but if
	// it is omitted, indexers will only be able to start indexing state from genesis.
	SyncSource decoding.SyncSource

	// Logger is the logger that indexers can use to write logs. It is optional.
	Logger logutil.Logger

	// Context is the context that indexers should use for shutdown signals via Context.Done(). It can also
	// be used to pass down other parameters to indexers if necessary. If it is omitted, context.Background
	// will be used.
	Context context.Context

	// AddressCodec is the address codec that indexers can use to encode and decode addresses. It should always be
	// provided, but if it is omitted, the indexer manager will use a default codec which encodes and decodes addresses
	// as hex strings.
	AddressCodec addressutil.AddressCodec

	// DoneWaitGroup is a wait group that all indexer manager go routines will wait on before returning when the context
	// is done.
	// It is optional.
	DoneWaitGroup *sync.WaitGroup
}

// IndexingConfig is the configuration of the indexer manager and contains the configuration for each indexer target.
type IndexingConfig struct {
	// Target is a map of named indexer targets to their configuration.
	Target map[string]Config

	// ChannelBufferSize is the buffer size of the channels used for buffering data sent to indexer go routines.
	// It defaults to 1024.
	ChannelBufferSize *int `json:"channel_buffer_size,omitempty"`
}

// IndexingTarget returns the indexing target listener and associated data.
// The returned listener is the root listener to which app data should be sent.
type IndexingTarget struct {
	// Listener is the root listener to which app data should be sent.
	// It will do all processing in the background so updates should be sent synchronously.
	Listener appdata.Listener

	// ModuleFilter returns the root module filter which an app can use to exclude modules at the storage level,
	// if such a filter is set.
	ModuleFilter *ModuleFilterConfig

	IndexerInfos map[string]IndexerInfo
}

// IndexerInfo contains data returned by a specific indexer after initialization that maybe useful for the app.
type IndexerInfo struct {
	// View is the view returned by the indexer in its InitResult. It is optional and may be nil.
	View view.AppData
}

// StartIndexing starts the indexer manager with the given options. The state machine should write all relevant app data to
// the returned listener.
func StartIndexing(opts IndexingOptions) (IndexingTarget, error) {
	logger := opts.Logger
	if logger == nil {
		logger = logutil.NoopLogger{}
	}

	logger.Info("Starting indexing")

	cfg, err := unmarshalIndexingConfig(opts.Config)
	if err != nil {
		return IndexingTarget{}, err
	}

	ctx := opts.Context
	if ctx == nil {
		ctx = context.Background()
	}

	listeners := make([]appdata.Listener, 0, len(cfg.Target))
	indexerInfos := make(map[string]IndexerInfo, len(cfg.Target))

	for targetName, targetCfg := range cfg.Target {
		init, ok := indexerRegistry[targetCfg.Type]
		if !ok {
			return IndexingTarget{}, fmt.Errorf("indexer type %q not found", targetCfg.Type)
		}

		logger.Info("Starting indexer", "target_name", targetName, "type", targetCfg.Type)

		if targetCfg.Filter != nil {
			return IndexingTarget{}, fmt.Errorf("indexer filter options are not supported yet")
		}

		childLogger := logger
		if scopeableLogger, ok := logger.(logutil.ScopeableLogger); ok {
			childLogger = scopeableLogger.WithContext("indexer", targetName).(logutil.Logger)
		}

		targetCfg.Config, err = unmarshalIndexerCustomConfig(targetCfg.Config, init.ConfigType)
		if err != nil {
			return IndexingTarget{}, fmt.Errorf("failed to unmarshal indexer config for target %q: %v", targetName, err)
		}

		initRes, err := init.InitFunc(InitParams{
			Config:       targetCfg,
			Context:      ctx,
			Logger:       childLogger,
			AddressCodec: opts.AddressCodec,
		})
		if err != nil {
			return IndexingTarget{}, err
		}

		listener := initRes.Listener
		listeners = append(listeners, listener)

		indexerInfos[targetName] = IndexerInfo{
			View: initRes.View,
		}
	}

	bufSize := 1024
	if cfg.ChannelBufferSize != nil {
		bufSize = *cfg.ChannelBufferSize
	}
	asyncOpts := appdata.AsyncListenerOptions{
		Context:       ctx,
		DoneWaitGroup: opts.DoneWaitGroup,
		BufferSize:    bufSize,
	}

	rootListener := appdata.AsyncListenerMux(
		asyncOpts,
		listeners...,
	)

	rootListener, err = decoding.Middleware(rootListener, opts.Resolver, decoding.MiddlewareOptions{})
	if err != nil {
		return IndexingTarget{}, err
	}
	rootListener = appdata.AsyncListener(asyncOpts, rootListener)

	return IndexingTarget{
		Listener:     rootListener,
		IndexerInfos: indexerInfos,
	}, nil
}

func unmarshalIndexingConfig(cfg interface{}) (*IndexingConfig, error) {
	if x, ok := cfg.(*IndexingConfig); ok {
		return x, nil
	}
	if x, ok := cfg.(IndexingConfig); ok {
		return &x, nil
	}

	var jsonBz []byte
	var err error

	switch cfg := cfg.(type) {
	case map[string]interface{}:
		jsonBz, err = json.Marshal(cfg)
		if err != nil {
			return nil, err
		}
	case json.RawMessage:
		jsonBz = cfg
	default:
		return nil, fmt.Errorf("can't convert %T to %T", cfg, IndexingConfig{})
	}

	var res IndexingConfig
	err = json.Unmarshal(jsonBz, &res)
	return &res, err
}

func unmarshalIndexerCustomConfig(cfg, expectedType interface{}) (interface{}, error) {
	typ := reflect.TypeOf(expectedType)
	if reflect.TypeOf(cfg).AssignableTo(typ) {
		return cfg, nil
	}

	res := reflect.New(typ).Interface()
	bz, err := json.Marshal(cfg)
	if err != nil {
		return nil, err
	}
	err = json.Unmarshal(bz, res)
	return reflect.ValueOf(res).Elem().Interface(), err
}

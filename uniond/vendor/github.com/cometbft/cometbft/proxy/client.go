package proxy

import (
	"fmt"

	abcicli "github.com/cometbft/cometbft/abci/client"
	"github.com/cometbft/cometbft/abci/example/kvstore"
	"github.com/cometbft/cometbft/abci/types"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	e2e "github.com/cometbft/cometbft/test/e2e/app"
)

//go:generate ../scripts/mockery_generate.sh ClientCreator

// ClientCreator creates new ABCI clients.
type ClientCreator interface {
	// NewABCIClient returns a new ABCI client.
	NewABCIClient() (abcicli.Client, error)
}

//----------------------------------------------------
// local proxy uses a mutex on an in-proc app

type localClientCreator struct {
	mtx *cmtsync.Mutex
	app types.Application
}

// NewLocalClientCreator returns a [ClientCreator] for the given app, which
// will be running locally.
//
// Maintains a single mutex over all new clients created with NewABCIClient. For
// a local client creator that uses a single mutex per new client, rather use
// [NewConnSyncLocalClientCreator].
func NewLocalClientCreator(app types.Application) ClientCreator {
	return &localClientCreator{
		mtx: new(cmtsync.Mutex),
		app: app,
	}
}

func (l *localClientCreator) NewABCIClient() (abcicli.Client, error) {
	return abcicli.NewLocalClient(l.mtx, l.app), nil
}

//----------------------------------------------------
// local proxy creates a new mutex for each client

type connSyncLocalClientCreator struct {
	app types.Application
}

// NewConnSyncLocalClientCreator returns a local [ClientCreator] for the given
// app.
//
// Unlike [NewLocalClientCreator], this is a "connection-synchronized" local
// client creator, meaning each call to NewABCIClient returns an ABCI client
// that maintains its own mutex over the application (i.e. it is
// per-"connection" synchronized).
func NewConnSyncLocalClientCreator(app types.Application) ClientCreator {
	return &connSyncLocalClientCreator{
		app: app,
	}
}

func (c *connSyncLocalClientCreator) NewABCIClient() (abcicli.Client, error) {
	// Specifying nil for the mutex causes each instance to create its own
	// mutex.
	return abcicli.NewLocalClient(nil, c.app), nil
}

//---------------------------------------------------------------
// remote proxy opens new connections to an external app process

type remoteClientCreator struct {
	addr        string
	transport   string
	mustConnect bool
}

// NewRemoteClientCreator returns a ClientCreator for the given address (e.g.
// "192.168.0.1") and transport (e.g. "tcp"). Set mustConnect to true if you
// want the client to connect before reporting success.
func NewRemoteClientCreator(addr, transport string, mustConnect bool) ClientCreator {
	return &remoteClientCreator{
		addr:        addr,
		transport:   transport,
		mustConnect: mustConnect,
	}
}

func (r *remoteClientCreator) NewABCIClient() (abcicli.Client, error) {
	remoteApp, err := abcicli.NewClient(r.addr, r.transport, r.mustConnect)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to proxy: %w", err)
	}

	return remoteApp, nil
}

// DefaultClientCreator returns a default [ClientCreator], which will create a
// local client if addr is one of "kvstore", "persistent_kvstore", "e2e",
// "noop".
//
// Otherwise a remote client will be created.
//
// Each of "kvstore", "persistent_kvstore" and "e2e" also currently have an
// "_connsync" variant (i.e. "kvstore_connsync", etc.), which attempts to
// replicate the same concurrency model as the remote client.
func DefaultClientCreator(addr, transport, dbDir string) ClientCreator {
	switch addr {
	case "kvstore":
		return NewLocalClientCreator(kvstore.NewInMemoryApplication())
	case "kvstore_connsync":
		return NewConnSyncLocalClientCreator(kvstore.NewInMemoryApplication())
	case "persistent_kvstore":
		return NewLocalClientCreator(kvstore.NewPersistentApplication(dbDir))
	case "persistent_kvstore_connsync":
		return NewConnSyncLocalClientCreator(kvstore.NewPersistentApplication(dbDir))
	case "e2e":
		app, err := e2e.NewApplication(e2e.DefaultConfig(dbDir))
		if err != nil {
			panic(err)
		}
		return NewLocalClientCreator(app)
	case "e2e_connsync":
		app, err := e2e.NewApplication(e2e.DefaultConfig(dbDir))
		if err != nil {
			panic(err)
		}
		return NewConnSyncLocalClientCreator(app)
	case "noop":
		return NewLocalClientCreator(types.NewBaseApplication())
	default:
		mustConnect := false // loop retrying
		return NewRemoteClientCreator(addr, transport, mustConnect)
	}
}

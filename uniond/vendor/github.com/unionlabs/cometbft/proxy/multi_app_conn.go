package proxy

import (
	"fmt"

	abcicli "github.com/cometbft/cometbft/abci/client"
	cmtos "github.com/cometbft/cometbft/internal/os"
	cmtlog "github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/service"
)

const (
	connConsensus = "consensus"
	connMempool   = "mempool"
	connQuery     = "query"
	connSnapshot  = "snapshot"
)

// AppConns is the CometBFT's interface to the application that consists of
// multiple connections.
type AppConns interface {
	service.Service

	// Mempool connection
	Mempool() AppConnMempool
	// Consensus connection
	Consensus() AppConnConsensus
	// Query connection
	Query() AppConnQuery
	// Snapshot connection
	Snapshot() AppConnSnapshot
}

// NewAppConns calls NewMultiAppConn.
func NewAppConns(clientCreator ClientCreator, metrics *Metrics) AppConns {
	return NewMultiAppConn(clientCreator, metrics)
}

// multiAppConn implements AppConns.
//
// A multiAppConn is made of a few appConns and manages their underlying abci
// clients.
// TODO: on app restart, clients must reboot together.
type multiAppConn struct {
	service.BaseService

	metrics       *Metrics
	consensusConn AppConnConsensus
	mempoolConn   AppConnMempool
	queryConn     AppConnQuery
	snapshotConn  AppConnSnapshot

	consensusConnClient abcicli.Client
	mempoolConnClient   abcicli.Client
	queryConnClient     abcicli.Client
	snapshotConnClient  abcicli.Client

	clientCreator ClientCreator
}

// NewMultiAppConn makes all necessary abci connections to the application.
func NewMultiAppConn(clientCreator ClientCreator, metrics *Metrics) AppConns {
	multiAppConn := &multiAppConn{
		metrics:       metrics,
		clientCreator: clientCreator,
	}
	multiAppConn.BaseService = *service.NewBaseService(nil, "multiAppConn", multiAppConn)
	return multiAppConn
}

func (app *multiAppConn) Mempool() AppConnMempool {
	return app.mempoolConn
}

func (app *multiAppConn) Consensus() AppConnConsensus {
	return app.consensusConn
}

func (app *multiAppConn) Query() AppConnQuery {
	return app.queryConn
}

func (app *multiAppConn) Snapshot() AppConnSnapshot {
	return app.snapshotConn
}

func (app *multiAppConn) OnStart() error {
	if err := app.startQueryClient(); err != nil {
		return err
	}
	if err := app.startSnapshotClient(); err != nil {
		app.stopAllClients()
		return err
	}
	if err := app.startMempoolClient(); err != nil {
		app.stopAllClients()
		return err
	}
	if err := app.startConsensusClient(); err != nil {
		app.stopAllClients()
		return err
	}

	// Kill CometBFT if the ABCI application crashes.
	go app.killTMOnClientError()

	return nil
}

func (app *multiAppConn) startQueryClient() error {
	c, err := app.clientCreator.NewABCIQueryClient()
	if err != nil {
		return fmt.Errorf("error creating ABCI client (query client): %w", err)
	}
	app.queryConnClient = c
	app.queryConn = NewAppConnQuery(c, app.metrics)
	return app.startClient(c, "query")
}

func (app *multiAppConn) startSnapshotClient() error {
	c, err := app.clientCreator.NewABCISnapshotClient()
	if err != nil {
		return fmt.Errorf("error creating ABCI client (snapshot client): %w", err)
	}
	app.snapshotConnClient = c
	app.snapshotConn = NewAppConnSnapshot(c, app.metrics)
	return app.startClient(c, "snapshot")
}

func (app *multiAppConn) startMempoolClient() error {
	c, err := app.clientCreator.NewABCIMempoolClient()
	if err != nil {
		return fmt.Errorf("error creating ABCI client (mempool client): %w", err)
	}
	app.mempoolConnClient = c
	app.mempoolConn = NewAppConnMempool(c, app.metrics)
	return app.startClient(c, "mempool")
}

func (app *multiAppConn) startConsensusClient() error {
	c, err := app.clientCreator.NewABCIConsensusClient()
	if err != nil {
		app.stopAllClients()
		return fmt.Errorf("error creating ABCI client (consensus client): %w", err)
	}
	app.consensusConnClient = c
	app.consensusConn = NewAppConnConsensus(c, app.metrics)
	return app.startClient(c, "consensus")
}

func (app *multiAppConn) startClient(c abcicli.Client, conn string) error {
	c.SetLogger(app.Logger.With("module", "abci-client", "connection", conn))
	if err := c.Start(); err != nil {
		return fmt.Errorf("error starting ABCI client (%s client): %w", conn, err)
	}
	return nil
}

func (app *multiAppConn) OnStop() {
	app.stopAllClients()
}

func (app *multiAppConn) killTMOnClientError() {
	killFn := func(conn string, err error, logger cmtlog.Logger) {
		logger.Error(
			conn+" connection terminated. Did the application crash? Please restart CometBFT",
			"err", err)
		killErr := cmtos.Kill()
		if killErr != nil {
			logger.Error("Failed to kill this process - please do so manually", "err", killErr)
		}
	}

	select {
	case <-app.consensusConnClient.Quit():
		if err := app.consensusConnClient.Error(); err != nil {
			killFn(connConsensus, err, app.Logger)
		}
	case <-app.mempoolConnClient.Quit():
		if err := app.mempoolConnClient.Error(); err != nil {
			killFn(connMempool, err, app.Logger)
		}
	case <-app.queryConnClient.Quit():
		if err := app.queryConnClient.Error(); err != nil {
			killFn(connQuery, err, app.Logger)
		}
	case <-app.snapshotConnClient.Quit():
		if err := app.snapshotConnClient.Error(); err != nil {
			killFn(connSnapshot, err, app.Logger)
		}
	}
}

func (app *multiAppConn) stopAllClients() {
	if app.consensusConnClient != nil {
		if err := app.consensusConnClient.Stop(); err != nil {
			app.Logger.Error("error while stopping consensus client", "error", err)
		}
	}
	if app.mempoolConnClient != nil {
		if err := app.mempoolConnClient.Stop(); err != nil {
			app.Logger.Error("error while stopping mempool client", "error", err)
		}
	}
	if app.queryConnClient != nil {
		if err := app.queryConnClient.Stop(); err != nil {
			app.Logger.Error("error while stopping query client", "error", err)
		}
	}
	if app.snapshotConnClient != nil {
		if err := app.snapshotConnClient.Stop(); err != nil {
			app.Logger.Error("error while stopping snapshot client", "error", err)
		}
	}
}

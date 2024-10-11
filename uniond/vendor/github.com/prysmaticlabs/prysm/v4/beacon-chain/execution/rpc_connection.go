package execution

import (
	"context"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/ethclient"
	gethRPC "github.com/ethereum/go-ethereum/rpc"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	contracts "github.com/prysmaticlabs/prysm/v4/contracts/deposit"
	"github.com/prysmaticlabs/prysm/v4/io/logs"
	"github.com/prysmaticlabs/prysm/v4/network"
	"github.com/prysmaticlabs/prysm/v4/network/authorization"
)

func (s *Service) setupExecutionClientConnections(ctx context.Context, currEndpoint network.Endpoint) error {
	client, err := s.newRPCClientWithAuth(ctx, currEndpoint)
	if err != nil {
		return errors.Wrap(err, "could not dial execution node")
	}
	// Attach the clients to the service struct.
	fetcher := ethclient.NewClient(client)
	s.rpcClient = client
	s.httpLogger = fetcher

	depositContractCaller, err := contracts.NewDepositContractCaller(s.cfg.depositContractAddr, fetcher)
	if err != nil {
		client.Close()
		return errors.Wrap(err, "could not initialize deposit contract caller")
	}
	s.depositContractCaller = depositContractCaller

	// Ensure we have the correct chain and deposit IDs.
	if err := ensureCorrectExecutionChain(ctx, fetcher); err != nil {
		client.Close()
		errStr := err.Error()
		if strings.Contains(errStr, "401 Unauthorized") {
			errStr = "could not verify execution chain ID as your connection is not authenticated. " +
				"If connecting to your execution client via HTTP, you will need to set up JWT authentication. " +
				"See our documentation here https://docs.prylabs.network/docs/execution-node/authentication"
		}
		return errors.Wrap(err, errStr)
	}
	s.updateConnectedETH1(true)
	s.runError = nil
	return nil
}

// Every N seconds, defined as a backoffPeriod, attempts to re-establish an execution client
// connection and if this does not work, we fallback to the next endpoint if defined.
func (s *Service) pollConnectionStatus(ctx context.Context) {
	// Use a custom logger to only log errors
	logCounter := 0
	errorLogger := func(err error, msg string) {
		if logCounter > logThreshold {
			log.WithError(err).Error(msg)
			logCounter = 0
		}
		logCounter++
	}
	ticker := time.NewTicker(backOffPeriod)
	defer ticker.Stop()
	for {
		select {
		case <-ticker.C:
			log.Debugf("Trying to dial endpoint: %s", logs.MaskCredentialsLogging(s.cfg.currHttpEndpoint.Url))
			currClient := s.rpcClient
			if err := s.setupExecutionClientConnections(ctx, s.cfg.currHttpEndpoint); err != nil {
				errorLogger(err, "Could not connect to execution client endpoint")
				continue
			}
			// Close previous client, if connection was successful.
			if currClient != nil {
				currClient.Close()
			}
			log.Infof("Connected to new endpoint: %s", logs.MaskCredentialsLogging(s.cfg.currHttpEndpoint.Url))
			return
		case <-s.ctx.Done():
			log.Debug("Received cancelled context,closing existing powchain service")
			return
		}
	}
}

// Forces to retry an execution client connection.
func (s *Service) retryExecutionClientConnection(ctx context.Context, err error) {
	s.runError = errors.Wrap(err, "retryExecutionClientConnection")
	s.updateConnectedETH1(false)
	// Back off for a while before redialing.
	time.Sleep(backOffPeriod)
	currClient := s.rpcClient
	if err := s.setupExecutionClientConnections(ctx, s.cfg.currHttpEndpoint); err != nil {
		s.runError = errors.Wrap(err, "setupExecutionClientConnections")
		return
	}
	// Close previous client, if connection was successful.
	if currClient != nil {
		currClient.Close()
	}
	// Reset run error in the event of a successful connection.
	s.runError = nil
}

// Initializes an RPC connection with authentication headers.
func (s *Service) newRPCClientWithAuth(ctx context.Context, endpoint network.Endpoint) (*gethRPC.Client, error) {
	headers := http.Header{}
	if endpoint.Auth.Method != authorization.None {
		header, err := endpoint.Auth.ToHeaderValue()
		if err != nil {
			return nil, err
		}
		headers.Set("Authorization", header)
	}
	for _, h := range s.cfg.headers {
		if h == "" {
			continue
		}
		keyValue := strings.Split(h, "=")
		if len(keyValue) < 2 {
			log.Warnf("Incorrect HTTP header flag format. Skipping %v", keyValue[0])
			continue
		}
		headers.Set(keyValue[0], strings.Join(keyValue[1:], "="))
	}
	return network.NewExecutionRPCClient(ctx, endpoint, headers)
}

// Checks the chain ID of the execution client to ensure
// it matches local parameters of what Prysm expects.
func ensureCorrectExecutionChain(ctx context.Context, client *ethclient.Client) error {
	cID, err := client.ChainID(ctx)
	if err != nil {
		return err
	}
	wantChainID := params.BeaconConfig().DepositChainID
	if cID.Uint64() != wantChainID {
		return fmt.Errorf("wanted chain ID %d, got %d", wantChainID, cID.Uint64())
	}
	return nil
}

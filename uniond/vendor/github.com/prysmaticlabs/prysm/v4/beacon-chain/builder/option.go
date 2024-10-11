package builder

import (
	"github.com/prysmaticlabs/prysm/v4/api/client/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/urfave/cli/v2"
)

type Option func(s *Service) error

// FlagOptions for builder service flag configurations.
func FlagOptions(c *cli.Context) ([]Option, error) {
	endpoint := c.String(flags.MevRelayEndpoint.Name)
	var client *builder.Client
	if endpoint != "" {
		var err error
		client, err = builder.NewClient(endpoint)
		if err != nil {
			return nil, err
		}
	}
	opts := []Option{
		WithBuilderClient(client),
	}
	return opts, nil
}

// WithBuilderClient sets the builder client for the beacon chain builder service.
func WithBuilderClient(client builder.BuilderClient) Option {
	return func(s *Service) error {
		s.cfg.builderClient = client
		return nil
	}
}

// WithHeadFetcher gets the head info from chain service.
func WithHeadFetcher(svc blockchain.HeadFetcher) Option {
	return func(s *Service) error {
		s.cfg.headFetcher = svc
		return nil
	}
}

// WithDatabase for head access.
func WithDatabase(beaconDB db.HeadAccessDatabase) Option {
	return func(s *Service) error {
		s.cfg.beaconDB = beaconDB
		return nil
	}
}

// WithRegistrationCache uses a cache for the validator registrations instead of a persistent db.
func WithRegistrationCache() Option {
	return func(s *Service) error {
		s.registrationCache = cache.NewRegistrationCache()
		return nil
	}
}

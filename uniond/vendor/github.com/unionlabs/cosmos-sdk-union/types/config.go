package types

import (
	"context"
	"sync"

	"github.com/cosmos/cosmos-sdk/version"
)

// DefaultKeyringServiceName defines a default service name for the keyring.
const DefaultKeyringServiceName = "cosmos"

func KeyringServiceName() string {
	if len(version.Name) == 0 {
		return DefaultKeyringServiceName
	}
	return version.Name
}

// Config is the structure that holds the SDK configuration parameters.
// Deprecated: The global SDK config is deprecated and users should prefer using an address codec.
// Users must still set the global config until the Stringer interface on `AccAddress`, `ValAddress`, and `ConsAddress` is removed.
type Config struct {
	bech32AddressPrefix map[string]string
	mtx                 sync.RWMutex

	sealed   bool
	sealedch chan struct{}
}

// cosmos-sdk wide global singleton
var (
	sdkConfig  *Config
	initConfig sync.Once
)

// New returns a new Config with default values.
func NewConfig() *Config {
	return &Config{
		sealedch: make(chan struct{}),
		bech32AddressPrefix: map[string]string{
			"account_addr":   Bech32PrefixAccAddr,
			"validator_addr": Bech32PrefixValAddr,
			"consensus_addr": Bech32PrefixConsAddr,
			"account_pub":    Bech32PrefixAccPub,
			"validator_pub":  Bech32PrefixValPub,
			"consensus_pub":  Bech32PrefixConsPub,
		},
	}
}

// GetConfig returns the config instance for the SDK.
func GetConfig() *Config {
	initConfig.Do(func() {
		sdkConfig = NewConfig()
	})
	return sdkConfig
}

// GetSealedConfig returns the config instance for the SDK if/once it is sealed.
func GetSealedConfig(ctx context.Context) (*Config, error) {
	config := GetConfig()
	select {
	case <-config.sealedch:
		return config, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

func (config *Config) assertNotSealed() {
	config.mtx.RLock()
	defer config.mtx.RUnlock()

	if config.sealed {
		panic("Config is sealed")
	}
}

// SetBech32PrefixForAccount builds the Config with Bech32 addressPrefix and publKeyPrefix for accounts
// and returns the config instance
func (config *Config) SetBech32PrefixForAccount(addressPrefix, pubKeyPrefix string) {
	config.assertNotSealed()
	config.bech32AddressPrefix["account_addr"] = addressPrefix
	config.bech32AddressPrefix["account_pub"] = pubKeyPrefix
}

// SetBech32PrefixForValidator builds the Config with Bech32 addressPrefix and publKeyPrefix for validators
//
//	and returns the config instance
func (config *Config) SetBech32PrefixForValidator(addressPrefix, pubKeyPrefix string) {
	config.assertNotSealed()
	config.bech32AddressPrefix["validator_addr"] = addressPrefix
	config.bech32AddressPrefix["validator_pub"] = pubKeyPrefix
}

// SetBech32PrefixForConsensusNode builds the Config with Bech32 addressPrefix and publKeyPrefix for consensus nodes
// and returns the config instance
func (config *Config) SetBech32PrefixForConsensusNode(addressPrefix, pubKeyPrefix string) {
	config.assertNotSealed()
	config.bech32AddressPrefix["consensus_addr"] = addressPrefix
	config.bech32AddressPrefix["consensus_pub"] = pubKeyPrefix
}

// Seal seals the config such that the config state could not be modified further
func (config *Config) Seal() *Config {
	config.mtx.Lock()

	if config.sealed {
		config.mtx.Unlock()
		return config
	}

	// signal sealed after state exposed/unlocked
	config.sealed = true
	config.mtx.Unlock()
	close(config.sealedch)

	return config
}

// GetBech32AccountAddrPrefix returns the Bech32 prefix for account address
func (config *Config) GetBech32AccountAddrPrefix() string {
	return config.bech32AddressPrefix["account_addr"]
}

// GetBech32ValidatorAddrPrefix returns the Bech32 prefix for validator address
func (config *Config) GetBech32ValidatorAddrPrefix() string {
	return config.bech32AddressPrefix["validator_addr"]
}

// GetBech32ConsensusAddrPrefix returns the Bech32 prefix for consensus node address
func (config *Config) GetBech32ConsensusAddrPrefix() string {
	return config.bech32AddressPrefix["consensus_addr"]
}

// GetBech32AccountPubPrefix returns the Bech32 prefix for account public key
func (config *Config) GetBech32AccountPubPrefix() string {
	return config.bech32AddressPrefix["account_pub"]
}

// GetBech32ValidatorPubPrefix returns the Bech32 prefix for validator public key
func (config *Config) GetBech32ValidatorPubPrefix() string {
	return config.bech32AddressPrefix["validator_pub"]
}

// GetBech32ConsensusPubPrefix returns the Bech32 prefix for consensus node public key
func (config *Config) GetBech32ConsensusPubPrefix() string {
	return config.bech32AddressPrefix["consensus_pub"]
}

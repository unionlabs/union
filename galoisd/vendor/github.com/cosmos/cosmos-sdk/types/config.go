package types

import (
	"context"
	"fmt"
	"sync"

	"github.com/cosmos/cosmos-sdk/version"
)

// DefaultKeyringServiceName defines a default service name for the keyring.
const DefaultKeyringServiceName = "cosmos"

// Config is the structure that holds the SDK configuration parameters.
// This could be used to initialize certain configuration parameters for the SDK.
type Config struct {
	fullFundraiserPath  string
	bech32AddressPrefix map[string]string
	txEncoder           TxEncoder
	addressVerifier     func([]byte) error
	mtx                 sync.RWMutex

	// SLIP-44 related
	purpose  uint32
	coinType uint32

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
		fullFundraiserPath: FullFundraiserPath,

		purpose:   Purpose,
		coinType:  CoinType,
		txEncoder: nil,
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
	config.mtx.Lock()
	defer config.mtx.Unlock()

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

// SetTxEncoder builds the Config with TxEncoder used to marshal StdTx to bytes
func (config *Config) SetTxEncoder(encoder TxEncoder) {
	config.assertNotSealed()
	config.txEncoder = encoder
}

// SetAddressVerifier builds the Config with the provided function for verifying that addresses
// have the correct format
func (config *Config) SetAddressVerifier(addressVerifier func([]byte) error) {
	config.assertNotSealed()
	config.addressVerifier = addressVerifier
}

// Set the FullFundraiserPath (BIP44Prefix) on the config.
//
// Deprecated: This method is supported for backward compatibility only and will be removed in a future release. Use SetPurpose and SetCoinType instead.
func (config *Config) SetFullFundraiserPath(fullFundraiserPath string) {
	config.assertNotSealed()
	config.fullFundraiserPath = fullFundraiserPath
}

// Set the BIP-0044 Purpose code on the config
func (config *Config) SetPurpose(purpose uint32) {
	config.assertNotSealed()
	config.purpose = purpose
}

// Set the BIP-0044 CoinType code on the config
func (config *Config) SetCoinType(coinType uint32) {
	config.assertNotSealed()
	config.coinType = coinType
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

// GetTxEncoder return function to encode transactions
func (config *Config) GetTxEncoder() TxEncoder {
	return config.txEncoder
}

// GetAddressVerifier returns the function to verify that addresses have the correct format
func (config *Config) GetAddressVerifier() func([]byte) error {
	return config.addressVerifier
}

// GetPurpose returns the BIP-0044 Purpose code on the config.
func (config *Config) GetPurpose() uint32 {
	return config.purpose
}

// GetCoinType returns the BIP-0044 CoinType code on the config.
func (config *Config) GetCoinType() uint32 {
	return config.coinType
}

// GetFullFundraiserPath returns the BIP44Prefix.
//
// Deprecated: This method is supported for backward compatibility only and will be removed in a future release. Use GetFullBIP44Path instead.
func (config *Config) GetFullFundraiserPath() string {
	return config.fullFundraiserPath
}

// GetFullBIP44Path returns the BIP44Prefix.
func (config *Config) GetFullBIP44Path() string {
	return fmt.Sprintf("m/%d'/%d'/0'/0/0", config.purpose, config.coinType)
}

func KeyringServiceName() string {
	if len(version.Name) == 0 {
		return DefaultKeyringServiceName
	}
	return version.Name
}

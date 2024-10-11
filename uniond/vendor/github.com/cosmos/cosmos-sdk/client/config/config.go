package config

import (
	"crypto/tls"
	"errors"
	"fmt"
	"os"
	"path/filepath"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
)

// DefaultConfig returns default config for the client.toml
func DefaultConfig() *Config {
	return &Config{
		ChainID:               "",
		KeyringBackend:        "os",
		KeyringDefaultKeyName: "",
		Output:                "text",
		Node:                  "tcp://localhost:26657",
		BroadcastMode:         "sync",
	}
}

// ClientConfig is an alias for Config for backward compatibility
// Deprecated: use Config instead which avoid name stuttering
type ClientConfig Config

type Config struct {
	ChainID               string     `mapstructure:"chain-id" json:"chain-id"`
	KeyringBackend        string     `mapstructure:"keyring-backend" json:"keyring-backend"`
	KeyringDefaultKeyName string     `mapstructure:"keyring-default-keyname" json:"keyring-default-keyname"`
	Output                string     `mapstructure:"output" json:"output"`
	Node                  string     `mapstructure:"node" json:"node"`
	BroadcastMode         string     `mapstructure:"broadcast-mode" json:"broadcast-mode"`
	GRPC                  GRPCConfig `mapstructure:",squash"`
}

// GRPCConfig holds the gRPC client configuration.
type GRPCConfig struct {
	Address  string `mapstructure:"grpc-address"  json:"grpc-address"`
	Insecure bool   `mapstructure:"grpc-insecure"  json:"grpc-insecure"`
}

// ReadFromClientConfig reads values from client.toml file and updates them in client.Context
// It uses CreateClientConfig internally with no custom template and custom config.
// Deprecated: use CreateClientConfig instead.
func ReadFromClientConfig(ctx client.Context) (client.Context, error) {
	return CreateClientConfig(ctx, "", nil)
}

// CreateClientConfig reads the client.toml file and returns a new populated client.Context
// If the client.toml file does not exist, it creates one with default values.
// It takes a customClientTemplate and customConfig as input that can be used to overwrite the default config and enhance the client.toml file.
// The custom template/config must be both provided or be "" and nil.
func CreateClientConfig(ctx client.Context, customClientTemplate string, customConfig interface{}) (client.Context, error) {
	configPath := filepath.Join(ctx.HomeDir, "config")
	configFilePath := filepath.Join(configPath, "client.toml")

	// when client.toml does not exist create and init with default values
	if _, err := os.Stat(configFilePath); os.IsNotExist(err) {
		if err := os.MkdirAll(configPath, os.ModePerm); err != nil {
			return ctx, fmt.Errorf("couldn't make client config: %w", err)
		}

		if (customClientTemplate != "" && customConfig == nil) || (customClientTemplate == "" && customConfig != nil) {
			return ctx, errors.New("customClientTemplate and customConfig should be both nil or not nil")
		}

		if customClientTemplate != "" {
			if err := setConfigTemplate(customClientTemplate); err != nil {
				return ctx, fmt.Errorf("couldn't set client config template: %w", err)
			}

			if ctx.ChainID != "" {
				// chain-id will be written to the client.toml while initiating the chain.
				ctx.Viper.Set(flags.FlagChainID, ctx.ChainID)
			}

			if err = ctx.Viper.Unmarshal(&customConfig); err != nil {
				return ctx, fmt.Errorf("failed to parse custom client config: %w", err)
			}

			if err := writeConfigFile(configFilePath, customConfig); err != nil {
				return ctx, fmt.Errorf("could not write client config to the file: %w", err)
			}

		} else {
			conf := DefaultConfig()
			if ctx.ChainID != "" {
				// chain-id will be written to the client.toml while initiating the chain.
				conf.ChainID = ctx.ChainID
			}

			if err := writeConfigFile(configFilePath, conf); err != nil {
				return ctx, fmt.Errorf("could not write client config to the file: %w", err)
			}
		}
	}

	conf, err := getClientConfig(configPath, ctx.Viper)
	if err != nil {
		return ctx, fmt.Errorf("couldn't get client config: %w", err)
	}

	// we need to update KeyringDir field on client.Context first cause it is used in NewKeyringFromBackend
	ctx = ctx.WithOutputFormat(conf.Output).
		WithChainID(conf.ChainID).
		WithKeyringDir(ctx.HomeDir).
		WithKeyringDefaultKeyName(conf.KeyringDefaultKeyName)

	keyring, err := client.NewKeyringFromBackend(ctx, conf.KeyringBackend)
	if err != nil {
		return ctx, fmt.Errorf("couldn't get keyring: %w", err)
	}

	// https://github.com/cosmos/cosmos-sdk/issues/8986
	client, err := client.NewClientFromNode(conf.Node)
	if err != nil {
		return ctx, fmt.Errorf("couldn't get client from nodeURI: %w", err)
	}

	ctx = ctx.
		WithNodeURI(conf.Node).
		WithBroadcastMode(conf.BroadcastMode).
		WithClient(client).
		WithKeyring(keyring)

	if conf.GRPC.Address != "" {
		grpcClient, err := getGRPCClient(conf.GRPC)
		if err != nil {
			return ctx, fmt.Errorf("couldn't get grpc client: %w", err)
		}

		ctx = ctx.WithGRPCClient(grpcClient)
	}

	return ctx, nil
}

// getGRPCClient creates and returns a new gRPC client connection based on the GRPCConfig.
// It determines the type of connection (secure or insecure) from the GRPCConfig and
// uses the specified server address to establish the connection.
func getGRPCClient(grpcConfig GRPCConfig) (*grpc.ClientConn, error) {
	transport := grpc.WithTransportCredentials(credentials.NewTLS(&tls.Config{
		MinVersion: tls.VersionTLS12,
	}))

	if grpcConfig.Insecure {
		transport = grpc.WithTransportCredentials(insecure.NewCredentials())
	}

	dialOptions := []grpc.DialOption{transport}
	grpcClient, err := grpc.NewClient(grpcConfig.Address, dialOptions...)
	if err != nil {
		return nil, fmt.Errorf("failed to dial gRPC server at %s: %w", grpcConfig.Address, err)
	}

	return grpcClient, nil
}

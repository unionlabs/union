package client

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path"
	"strings"

	"github.com/cosmos/gogoproto/proto"
	"github.com/spf13/viper"
	"google.golang.org/grpc"
	"sigs.k8s.io/yaml"

	"cosmossdk.io/core/address"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// PreprocessTxFn defines a hook by which chains can preprocess transactions before broadcasting
type PreprocessTxFn func(chainID string, key keyring.KeyType, tx TxBuilder) error

// Context implements a typical context created in SDK modules for transaction
// handling and queries.
type Context struct {
	Client                CometRPC
	GRPCClient            *grpc.ClientConn
	ChainID               string
	Codec                 codec.Codec
	InterfaceRegistry     codectypes.InterfaceRegistry
	Input                 io.Reader
	Keyring               keyring.Keyring
	KeyringOptions        []keyring.Option
	KeyringDir            string
	KeyringDefaultKeyName string
	Output                io.Writer
	OutputFormat          string
	Height                int64
	HomeDir               string
	// From is a name or an address of a keyring account used to set FromName and FromAddress fields.
	// Should be set by the "from" flag.
	From string
	// Name of a keyring account used to sign transactions.
	FromName string
	// Address of a keyring account used to sign transactions.
	FromAddress       sdk.AccAddress
	BroadcastMode     string
	SignModeStr       string
	UseLedger         bool
	Simulate          bool
	GenerateOnly      bool
	Offline           bool
	SkipConfirm       bool
	TxConfig          TxConfig
	AccountRetriever  AccountRetriever
	NodeURI           string
	FeePayer          sdk.AccAddress
	FeeGranter        sdk.AccAddress
	Viper             *viper.Viper
	LedgerHasProtobuf bool
	PreprocessTxHook  PreprocessTxFn

	// IsAux is true when the signer is an auxiliary signer (e.g. the tipper).
	IsAux bool

	// TODO: Deprecated (remove).
	LegacyAmino *codec.LegacyAmino

	// CmdContext is the context.Context from the Cobra command.
	CmdContext context.Context

	// Address codecs
	AddressCodec          address.Codec
	ValidatorAddressCodec address.Codec
	ConsensusAddressCodec address.Codec

	// Bech32 address prefixes.
	AddressPrefix   string
	ValidatorPrefix string
}

// WithCmdContext returns a copy of the context with an updated context.Context,
// usually set to the cobra cmd context.
func (ctx Context) WithCmdContext(c context.Context) Context {
	ctx.CmdContext = c
	return ctx
}

// WithKeyring returns a copy of the context with an updated keyring.
func (ctx Context) WithKeyring(k keyring.Keyring) Context {
	ctx.Keyring = k
	return ctx
}

// WithKeyringOptions returns a copy of the context with an updated keyring.
func (ctx Context) WithKeyringOptions(opts ...keyring.Option) Context {
	ctx.KeyringOptions = opts
	return ctx
}

// WithInput returns a copy of the context with an updated input.
func (ctx Context) WithInput(r io.Reader) Context {
	// convert to a bufio.Reader to have a shared buffer between the keyring and the
	// Commands, ensuring a read from one advance the read pointer for the other.
	// see https://github.com/cosmos/cosmos-sdk/issues/9566.
	ctx.Input = bufio.NewReader(r)
	return ctx
}

// WithCodec returns a copy of the Context with an updated Codec.
func (ctx Context) WithCodec(m codec.Codec) Context {
	ctx.Codec = m
	return ctx
}

// WithLegacyAmino returns a copy of the context with an updated LegacyAmino codec.
// TODO: Deprecated (remove).
func (ctx Context) WithLegacyAmino(cdc *codec.LegacyAmino) Context {
	ctx.LegacyAmino = cdc
	return ctx
}

// WithOutput returns a copy of the context with an updated output writer (e.g. stdout).
func (ctx Context) WithOutput(w io.Writer) Context {
	ctx.Output = w
	return ctx
}

// WithFrom returns a copy of the context with an updated from address or name.
func (ctx Context) WithFrom(from string) Context {
	ctx.From = from
	return ctx
}

// WithOutputFormat returns a copy of the context with an updated OutputFormat field.
func (ctx Context) WithOutputFormat(format string) Context {
	ctx.OutputFormat = format
	return ctx
}

// WithNodeURI returns a copy of the context with an updated node URI.
func (ctx Context) WithNodeURI(nodeURI string) Context {
	ctx.NodeURI = nodeURI
	return ctx
}

// WithHeight returns a copy of the context with an updated height.
func (ctx Context) WithHeight(height int64) Context {
	ctx.Height = height
	return ctx
}

// WithClient returns a copy of the context with an updated RPC client
// instance.
func (ctx Context) WithClient(client CometRPC) Context {
	ctx.Client = client
	return ctx
}

// WithGRPCClient returns a copy of the context with an updated GRPC client
// instance.
func (ctx Context) WithGRPCClient(grpcClient *grpc.ClientConn) Context {
	ctx.GRPCClient = grpcClient
	return ctx
}

// WithUseLedger returns a copy of the context with an updated UseLedger flag.
func (ctx Context) WithUseLedger(useLedger bool) Context {
	ctx.UseLedger = useLedger
	return ctx
}

// WithChainID returns a copy of the context with an updated chain ID.
func (ctx Context) WithChainID(chainID string) Context {
	ctx.ChainID = chainID
	return ctx
}

// WithHomeDir returns a copy of the Context with HomeDir set.
func (ctx Context) WithHomeDir(dir string) Context {
	if dir != "" {
		ctx.HomeDir = dir
	}
	return ctx
}

// WithKeyringDir returns a copy of the Context with KeyringDir set.
func (ctx Context) WithKeyringDir(dir string) Context {
	ctx.KeyringDir = dir
	return ctx
}

// WithKeyringDefaultKeyName returns a copy of the Context with KeyringDefaultKeyName set.
func (ctx Context) WithKeyringDefaultKeyName(keyName string) Context {
	ctx.KeyringDefaultKeyName = keyName
	return ctx
}

// WithGenerateOnly returns a copy of the context with updated GenerateOnly value
func (ctx Context) WithGenerateOnly(generateOnly bool) Context {
	ctx.GenerateOnly = generateOnly
	return ctx
}

// WithSimulation returns a copy of the context with updated Simulate value
func (ctx Context) WithSimulation(simulate bool) Context {
	ctx.Simulate = simulate
	return ctx
}

// WithOffline returns a copy of the context with updated Offline value.
func (ctx Context) WithOffline(offline bool) Context {
	ctx.Offline = offline
	return ctx
}

// WithFromName returns a copy of the context with an updated from account name.
func (ctx Context) WithFromName(name string) Context {
	ctx.FromName = name
	return ctx
}

// WithFromAddress returns a copy of the context with an updated from account
// address.
func (ctx Context) WithFromAddress(addr sdk.AccAddress) Context {
	ctx.FromAddress = addr
	return ctx
}

// WithFeePayerAddress returns a copy of the context with an updated fee payer account
// address.
func (ctx Context) WithFeePayerAddress(addr sdk.AccAddress) Context {
	ctx.FeePayer = addr
	return ctx
}

// WithFeeGranterAddress returns a copy of the context with an updated fee granter account
// address.
func (ctx Context) WithFeeGranterAddress(addr sdk.AccAddress) Context {
	ctx.FeeGranter = addr
	return ctx
}

// WithBroadcastMode returns a copy of the context with an updated broadcast
// mode.
func (ctx Context) WithBroadcastMode(mode string) Context {
	ctx.BroadcastMode = mode
	return ctx
}

// WithSignModeStr returns a copy of the context with an updated SignMode
// value.
func (ctx Context) WithSignModeStr(signModeStr string) Context {
	ctx.SignModeStr = signModeStr
	return ctx
}

// WithSkipConfirmation returns a copy of the context with an updated SkipConfirm
// value.
func (ctx Context) WithSkipConfirmation(skip bool) Context {
	ctx.SkipConfirm = skip
	return ctx
}

// WithTxConfig returns the context with an updated TxConfig
func (ctx Context) WithTxConfig(generator TxConfig) Context {
	ctx.TxConfig = generator
	return ctx
}

// WithAccountRetriever returns the context with an updated AccountRetriever
func (ctx Context) WithAccountRetriever(retriever AccountRetriever) Context {
	ctx.AccountRetriever = retriever
	return ctx
}

// WithInterfaceRegistry returns the context with an updated InterfaceRegistry
func (ctx Context) WithInterfaceRegistry(interfaceRegistry codectypes.InterfaceRegistry) Context {
	ctx.InterfaceRegistry = interfaceRegistry
	return ctx
}

// WithViper returns the context with Viper field. This Viper instance is used to read
// client-side config from the config file.
func (ctx Context) WithViper(prefix string) Context {
	v := viper.New()

	if prefix == "" {
		executableName, _ := os.Executable()
		prefix = path.Base(executableName)
	}

	v.SetEnvPrefix(prefix)
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_", "-", "_"))
	v.AutomaticEnv()
	ctx.Viper = v
	return ctx
}

// WithAux returns a copy of the context with an updated IsAux value.
func (ctx Context) WithAux(isAux bool) Context {
	ctx.IsAux = isAux
	return ctx
}

// WithLedgerHasProtobuf returns the context with the provided boolean value, indicating
// whether the target Ledger application can support Protobuf payloads.
func (ctx Context) WithLedgerHasProtobuf(val bool) Context {
	ctx.LedgerHasProtobuf = val
	return ctx
}

// WithPreprocessTxHook returns the context with the provided preprocessing hook, which
// enables chains to preprocess the transaction using the builder.
func (ctx Context) WithPreprocessTxHook(preprocessFn PreprocessTxFn) Context {
	ctx.PreprocessTxHook = preprocessFn
	return ctx
}

// WithAddressCodec returns the context with the provided address codec.
func (ctx Context) WithAddressCodec(addressCodec address.Codec) Context {
	ctx.AddressCodec = addressCodec
	return ctx
}

// WithValidatorAddressCodec returns the context with the provided validator address codec.
func (ctx Context) WithValidatorAddressCodec(validatorAddressCodec address.Codec) Context {
	ctx.ValidatorAddressCodec = validatorAddressCodec
	return ctx
}

// WithConsensusAddressCodec returns the context with the provided consensus address codec.
func (ctx Context) WithConsensusAddressCodec(consensusAddressCodec address.Codec) Context {
	ctx.ConsensusAddressCodec = consensusAddressCodec
	return ctx
}

// WithAddressPrefix returns the context with the provided address bech32 prefix.
func (ctx Context) WithAddressPrefix(addressPrefix string) Context {
	ctx.AddressPrefix = addressPrefix
	return ctx
}

// WithValidatorPrefix returns the context with the provided validator bech32 prefix.
func (ctx Context) WithValidatorPrefix(validatorPrefix string) Context {
	ctx.ValidatorPrefix = validatorPrefix
	return ctx
}

// PrintString prints the raw string to ctx.Output if it's defined, otherwise to os.Stdout
func (ctx Context) PrintString(str string) error {
	return ctx.PrintBytes([]byte(str))
}

// PrintBytes prints the raw bytes to ctx.Output if it's defined, otherwise to os.Stdout.
// NOTE: for printing a complex state object, you should use ctx.PrintOutput
func (ctx Context) PrintBytes(o []byte) error {
	writer := ctx.Output
	if writer == nil {
		writer = os.Stdout
	}

	_, err := writer.Write(o)
	return err
}

// PrintProto outputs toPrint to the ctx.Output based on ctx.OutputFormat which is
// either text or json. If text, toPrint will be YAML encoded. Otherwise, toPrint
// will be JSON encoded using ctx.Codec. An error is returned upon failure.
func (ctx Context) PrintProto(toPrint proto.Message) error {
	// always serialize JSON initially because proto json can't be directly YAML encoded
	out, err := ctx.Codec.MarshalJSON(toPrint)
	if err != nil {
		return err
	}
	return ctx.printOutput(out)
}

// PrintRaw is a variant of PrintProto that doesn't require a proto.Message type
// and uses a raw JSON message. No marshaling is performed.
func (ctx Context) PrintRaw(toPrint json.RawMessage) error {
	return ctx.printOutput(toPrint)
}

func (ctx Context) printOutput(out []byte) error {
	var err error
	if ctx.OutputFormat == "text" {
		out, err = yaml.JSONToYAML(out)
		if err != nil {
			return err
		}
	}

	writer := ctx.Output
	if writer == nil {
		writer = os.Stdout
	}

	_, err = writer.Write(out)
	if err != nil {
		return err
	}

	if ctx.OutputFormat != "text" {
		// append new-line for formats besides YAML
		_, err = writer.Write([]byte("\n"))
		if err != nil {
			return err
		}
	}

	return nil
}

// GetFromFields returns a from account address, account name and keyring type, given either an address or key name.
// If clientCtx.Simulate is true the keystore is not accessed and a valid address must be provided
// If clientCtx.GenerateOnly is true the keystore is only accessed if a key name is provided
// If from is empty, the default key if specified in the context will be used
func GetFromFields(clientCtx Context, kr keyring.Keyring, from string) (sdk.AccAddress, string, keyring.KeyType, error) {
	if from == "" && clientCtx.KeyringDefaultKeyName != "" {
		from = clientCtx.KeyringDefaultKeyName
		_ = clientCtx.PrintString(fmt.Sprintf("No key name or address provided; using the default key: %s\n", clientCtx.KeyringDefaultKeyName))
	}

	if from == "" {
		return nil, "", 0, nil
	}

	addr, err := clientCtx.AddressCodec.StringToBytes(from)
	switch {
	case clientCtx.Simulate:
		if err != nil {
			return nil, "", 0, fmt.Errorf("a valid address must be provided in simulation mode: %w", err)
		}

		return addr, "", 0, nil

	case clientCtx.GenerateOnly:
		if err == nil {
			return addr, "", 0, nil
		}
	}

	var k *keyring.Record
	if err == nil {
		k, err = kr.KeyByAddress(addr)
		if err != nil {
			return nil, "", 0, err
		}
	} else {
		k, err = kr.Key(from)
		if err != nil {
			return nil, "", 0, err
		}
	}

	addr, err = k.GetAddress()
	if err != nil {
		return nil, "", 0, err
	}

	return addr, k.Name, k.GetType(), nil
}

// NewKeyringFromBackend gets a Keyring object from a backend
func NewKeyringFromBackend(ctx Context, backend string) (keyring.Keyring, error) {
	if ctx.Simulate {
		backend = keyring.BackendMemory
	}

	return keyring.New(sdk.KeyringServiceName(), backend, ctx.KeyringDir, ctx.Input, ctx.Codec, ctx.KeyringOptions...)
}

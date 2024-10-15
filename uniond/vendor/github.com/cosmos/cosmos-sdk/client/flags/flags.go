package flags

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"
	"github.com/spf13/pflag"

	"github.com/cosmos/cosmos-sdk/crypto/keyring"
)

const (
	// DefaultGasAdjustment is applied to gas estimates to avoid tx execution
	// failures due to state changes that might occur between the tx simulation
	// and the actual run.
	DefaultGasAdjustment = 1.0
	DefaultGasLimit      = 200000
	GasFlagAuto          = "auto"

	// DefaultKeyringBackend defines the default keyring backend to be used
	DefaultKeyringBackend = keyring.BackendOS

	// BroadcastSync defines a tx broadcasting mode where the client waits for
	// a CheckTx execution response only.
	BroadcastSync = "sync"
	// BroadcastAsync defines a tx broadcasting mode where the client returns
	// immediately.
	BroadcastAsync = "async"

	// SignModeDirect is the value of the --sign-mode flag for SIGN_MODE_DIRECT
	SignModeDirect = "direct"
	// SignModeLegacyAminoJSON is the value of the --sign-mode flag for SIGN_MODE_LEGACY_AMINO_JSON
	SignModeLegacyAminoJSON = "amino-json"
	// SignModeDirectAux is the value of the --sign-mode flag for SIGN_MODE_DIRECT_AUX
	SignModeDirectAux = "direct-aux"
	// SignModeTextual is the value of the --sign-mode flag for SIGN_MODE_TEXTUAL.
	SignModeTextual = "textual"
	// SignModeEIP191 is the value of the --sign-mode flag for SIGN_MODE_EIP_191
	SignModeEIP191 = "eip-191"
)

// List of CLI flags
const (
	FlagHome             = "home"
	FlagKeyringDir       = "keyring-dir"
	FlagUseLedger        = "ledger"
	FlagChainID          = "chain-id"
	FlagNode             = "node"
	FlagGRPC             = "grpc-addr"
	FlagGRPCInsecure     = "grpc-insecure"
	FlagHeight           = "height"
	FlagGasAdjustment    = "gas-adjustment"
	FlagFrom             = "from"
	FlagName             = "name"
	FlagAccountNumber    = "account-number"
	FlagSequence         = "sequence"
	FlagNote             = "note"
	FlagFees             = "fees"
	FlagGas              = "gas"
	FlagGasPrices        = "gas-prices"
	FlagBroadcastMode    = "broadcast-mode"
	FlagDryRun           = "dry-run"
	FlagGenerateOnly     = "generate-only"
	FlagOffline          = "offline"
	FlagOutputDocument   = "output-document" // inspired by wget -O
	FlagSkipConfirmation = "yes"
	FlagProve            = "prove"
	FlagKeyringBackend   = "keyring-backend"
	FlagPage             = "page"
	FlagLimit            = "limit"
	FlagSignMode         = "sign-mode"
	FlagPageKey          = "page-key"
	FlagOffset           = "offset"
	FlagCountTotal       = "count-total"
	FlagTimeoutTimestamp = "timeout-timestamp"
	FlagUnordered        = "unordered"
	FlagKeyAlgorithm     = "algo"
	FlagKeyType          = "key-type"
	FlagFeePayer         = "fee-payer"
	FlagFeeGranter       = "fee-granter"
	FlagReverse          = "reverse"
	FlagTip              = "tip"
	FlagAux              = "aux"
	FlagInitHeight       = "initial-height"
	// FlagOutput is the flag to set the output format.
	// This differs from FlagOutputDocument that is used to set the output file.
	FlagOutput = "output"
	// FlagLogLevel defines the flag for setting the log level
	FlagLogLevel   = "log_level"
	FlagLogFormat  = "log_format"
	FlagLogNoColor = "log_no_color"
)

// List of supported output formats
const (
	OutputFormatJSON = "json"
	OutputFormatText = "text"
)

// LineBreak can be included in a command list to provide a blank line
// to help with readability
var LineBreak = &cobra.Command{Run: func(*cobra.Command, []string) {}}

// AddQueryFlagsToCmd adds common flags to a module query command.
func AddQueryFlagsToCmd(cmd *cobra.Command) {
	cmd.Flags().String(FlagNode, "tcp://localhost:26657", "<host>:<port> to CometBFT RPC interface for this chain")
	cmd.Flags().String(FlagGRPC, "", "the gRPC endpoint to use for this chain")
	cmd.Flags().Bool(FlagGRPCInsecure, false, "allow gRPC over insecure channels, if not the server must use TLS")
	cmd.Flags().Int64(FlagHeight, 0, "Use a specific height to query state at (this can error if the node is pruning state)")
	cmd.Flags().StringP(FlagOutput, "o", "text", "Output format (text|json)")

	// some base commands does not require chainID e.g `simd testnet` while subcommands do
	// hence the flag should not be required for those commands
	_ = cmd.MarkFlagRequired(FlagChainID)
}

// AddTxFlagsToCmd adds common flags to a module tx command.
func AddTxFlagsToCmd(cmd *cobra.Command) {
	f := cmd.Flags()
	f.StringP(FlagOutput, "o", OutputFormatJSON, "Output format (text|json)")
	if cmd.Flag(FlagFrom) == nil { // avoid flag redefinition when it's already been added by AutoCLI
		f.String(FlagFrom, "", "Name or address of private key with which to sign")
	}
	f.Uint64P(FlagAccountNumber, "a", 0, "The account number of the signing account (offline mode only)")
	f.Uint64P(FlagSequence, "s", 0, "The sequence number of the signing account (offline mode only)")
	f.String(FlagNote, "", "Note to add a description to the transaction (previously --memo)")
	f.String(FlagFees, "", "Fees to pay along with transaction; eg: 10uatom")
	f.String(FlagGasPrices, "", "Determine the transaction fee by multiplying max gas units by gas prices (e.g. 0.1uatom), rounding up to nearest denom unit")
	f.String(FlagNode, "tcp://localhost:26657", "<host>:<port> to CometBFT rpc interface for this chain")
	f.Bool(FlagUseLedger, false, "Use a connected Ledger device")
	f.Float64(FlagGasAdjustment, DefaultGasAdjustment, "adjustment factor to be multiplied against the estimate returned by the tx simulation; if the gas limit is set manually this flag is ignored ")
	f.StringP(FlagBroadcastMode, "b", BroadcastSync, "Transaction broadcasting mode (sync|async)")
	f.Bool(FlagDryRun, false, "ignore the --gas flag and perform a simulation of a transaction, but don't broadcast it (when enabled, the local Keybase is not accessible)")
	f.Bool(FlagGenerateOnly, false, "Build an unsigned transaction and write it to STDOUT (when enabled, the local Keybase only accessed when providing a key name)")
	f.Bool(FlagOffline, false, "Offline mode (does not allow any online functionality)")
	f.BoolP(FlagSkipConfirmation, "y", false, "Skip tx broadcasting prompt confirmation")
	f.String(FlagSignMode, "", "Choose sign mode (direct|amino-json|direct-aux|textual), this is an advanced feature")
	f.Int64(FlagTimeoutTimestamp, 0, "Set a block timeout timestamp to prevent the tx from being committed past a certain time")
	f.Bool(FlagUnordered, false, "Enable unordered transaction delivery; must be used in conjunction with --timeout-timestamp")
	f.String(FlagFeePayer, "", "Fee payer pays fees for the transaction instead of deducting from the signer")
	f.String(FlagFeeGranter, "", "Fee granter grants fees for the transaction")
	f.String(FlagTip, "", "Tip is the amount that is going to be transferred to the fee payer on the target chain. This flag is only valid when used with --aux, and is ignored if the target chain didn't enable the TipDecorator")
	f.Bool(FlagAux, false, "Generate aux signer data instead of sending a tx")
	f.String(FlagChainID, "", "The network chain ID")
	// --gas can accept integers and "auto"
	f.String(FlagGas, "", fmt.Sprintf("gas limit to set per-transaction; set to %q to calculate sufficient gas automatically. Note: %q option doesn't always report accurate results. Set a valid coin value to adjust the result. Can be used instead of %q. (default %d)",
		GasFlagAuto, GasFlagAuto, FlagFees, DefaultGasLimit))

	AddKeyringFlags(f)
}

// AddKeyringFlags sets common keyring flags
func AddKeyringFlags(flags *pflag.FlagSet) {
	flags.String(FlagKeyringDir, "", "The client Keyring directory; if omitted, the default 'home' directory will be used")
	flags.String(FlagKeyringBackend, DefaultKeyringBackend, "Select keyring's backend (os|file|kwallet|pass|test|memory)")
}

// AddPaginationFlagsToCmd adds common pagination flags to cmd
func AddPaginationFlagsToCmd(cmd *cobra.Command, query string) {
	cmd.Flags().Uint64(FlagPage, 1, fmt.Sprintf("pagination page of %s to query for. This sets offset to a multiple of limit", query))
	cmd.Flags().String(FlagPageKey, "", fmt.Sprintf("pagination page-key of %s to query for", query))
	cmd.Flags().Uint64(FlagOffset, 0, fmt.Sprintf("pagination offset of %s to query for", query))
	cmd.Flags().Uint64(FlagLimit, 100, fmt.Sprintf("pagination limit of %s to query for", query))
	cmd.Flags().Bool(FlagCountTotal, false, fmt.Sprintf("count total number of records in %s to query for", query))
	cmd.Flags().Bool(FlagReverse, false, "results are sorted in descending order")
}

// GasSetting encapsulates the possible values passed through the --gas flag.
type GasSetting struct {
	Simulate bool
	Gas      uint64
}

func (v *GasSetting) String() string {
	if v.Simulate {
		return GasFlagAuto
	}

	return strconv.FormatUint(v.Gas, 10)
}

// ParseGasSetting parses a string gas value. The value may either be 'auto',
// which indicates a transaction should be executed in simulate mode to
// automatically find a sufficient gas value, or a string integer. It returns an
// error if a string integer is provided which cannot be parsed.
func ParseGasSetting(gasStr string) (GasSetting, error) {
	switch gasStr {
	case "":
		return GasSetting{false, DefaultGasLimit}, nil

	case GasFlagAuto:
		return GasSetting{true, 0}, nil

	default:
		gas, err := strconv.ParseUint(gasStr, 10, 64)
		if err != nil {
			return GasSetting{}, fmt.Errorf("gas must be either integer or %s", GasFlagAuto)
		}

		return GasSetting{false, gas}, nil
	}
}

package tx

import (
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/cosmos/go-bip39"
	"github.com/spf13/pflag"

	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// Factory defines a client transaction factory that facilitates generating and
// signing an application-specific transaction.
type Factory struct {
	keybase            keyring.Keyring
	txConfig           client.TxConfig
	accountRetriever   client.AccountRetriever
	accountNumber      uint64
	sequence           uint64
	gas                uint64
	timeoutHeight      uint64
	gasAdjustment      float64
	chainID            string
	fromName           string
	offline            bool
	generateOnly       bool
	memo               string
	fees               sdk.Coins
	feeGranter         sdk.AccAddress
	feePayer           sdk.AccAddress
	gasPrices          sdk.DecCoins
	extOptions         []*codectypes.Any
	signMode           signing.SignMode
	simulateAndExecute bool
	preprocessTxHook   client.PreprocessTxFn
}

// NewFactoryCLI creates a new Factory.
func NewFactoryCLI(clientCtx client.Context, flagSet *pflag.FlagSet) (Factory, error) {
	signMode := signing.SignMode_SIGN_MODE_UNSPECIFIED
	switch clientCtx.SignModeStr {
	case flags.SignModeDirect:
		signMode = signing.SignMode_SIGN_MODE_DIRECT
	case flags.SignModeLegacyAminoJSON:
		signMode = signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON
	case flags.SignModeDirectAux:
		signMode = signing.SignMode_SIGN_MODE_DIRECT_AUX
	case flags.SignModeTextual:
		signMode = signing.SignMode_SIGN_MODE_TEXTUAL
	case flags.SignModeEIP191:
		signMode = signing.SignMode_SIGN_MODE_EIP_191
	}

	var accNum, accSeq uint64
	if clientCtx.Offline {
		if flagSet.Changed(flags.FlagAccountNumber) && flagSet.Changed(flags.FlagSequence) {
			accNum, _ = flagSet.GetUint64(flags.FlagAccountNumber)
			accSeq, _ = flagSet.GetUint64(flags.FlagSequence)
		} else {
			return Factory{}, errors.New("account-number and sequence must be set in offline mode")
		}
	}

	gasAdj, _ := flagSet.GetFloat64(flags.FlagGasAdjustment)
	memo, _ := flagSet.GetString(flags.FlagNote)
	timeoutHeight, _ := flagSet.GetUint64(flags.FlagTimeoutHeight)

	gasStr, _ := flagSet.GetString(flags.FlagGas)
	gasSetting, _ := flags.ParseGasSetting(gasStr)

	f := Factory{
		txConfig:           clientCtx.TxConfig,
		accountRetriever:   clientCtx.AccountRetriever,
		keybase:            clientCtx.Keyring,
		chainID:            clientCtx.ChainID,
		fromName:           clientCtx.FromName,
		offline:            clientCtx.Offline,
		generateOnly:       clientCtx.GenerateOnly,
		gas:                gasSetting.Gas,
		simulateAndExecute: gasSetting.Simulate,
		accountNumber:      accNum,
		sequence:           accSeq,
		timeoutHeight:      timeoutHeight,
		gasAdjustment:      gasAdj,
		memo:               memo,
		signMode:           signMode,
		feeGranter:         clientCtx.FeeGranter,
		feePayer:           clientCtx.FeePayer,
	}

	feesStr, _ := flagSet.GetString(flags.FlagFees)
	f = f.WithFees(feesStr)

	gasPricesStr, _ := flagSet.GetString(flags.FlagGasPrices)
	f = f.WithGasPrices(gasPricesStr)

	f = f.WithPreprocessTxHook(clientCtx.PreprocessTxHook)

	return f, nil
}

func (f Factory) AccountNumber() uint64                     { return f.accountNumber }
func (f Factory) Sequence() uint64                          { return f.sequence }
func (f Factory) Gas() uint64                               { return f.gas }
func (f Factory) GasAdjustment() float64                    { return f.gasAdjustment }
func (f Factory) Keybase() keyring.Keyring                  { return f.keybase }
func (f Factory) ChainID() string                           { return f.chainID }
func (f Factory) Memo() string                              { return f.memo }
func (f Factory) Fees() sdk.Coins                           { return f.fees }
func (f Factory) GasPrices() sdk.DecCoins                   { return f.gasPrices }
func (f Factory) AccountRetriever() client.AccountRetriever { return f.accountRetriever }
func (f Factory) TimeoutHeight() uint64                     { return f.timeoutHeight }
func (f Factory) FromName() string                          { return f.fromName }

// SimulateAndExecute returns the option to simulate and then execute the transaction
// using the gas from the simulation results
func (f Factory) SimulateAndExecute() bool { return f.simulateAndExecute }

// WithTxConfig returns a copy of the Factory with an updated TxConfig.
func (f Factory) WithTxConfig(g client.TxConfig) Factory {
	f.txConfig = g
	return f
}

// WithAccountRetriever returns a copy of the Factory with an updated AccountRetriever.
func (f Factory) WithAccountRetriever(ar client.AccountRetriever) Factory {
	f.accountRetriever = ar
	return f
}

// WithChainID returns a copy of the Factory with an updated chainID.
func (f Factory) WithChainID(chainID string) Factory {
	f.chainID = chainID
	return f
}

// WithGas returns a copy of the Factory with an updated gas value.
func (f Factory) WithGas(gas uint64) Factory {
	f.gas = gas
	return f
}

// WithFees returns a copy of the Factory with an updated fee.
func (f Factory) WithFees(fees string) Factory {
	parsedFees, err := sdk.ParseCoinsNormalized(fees)
	if err != nil {
		panic(err)
	}

	f.fees = parsedFees
	return f
}

// WithGasPrices returns a copy of the Factory with updated gas prices.
func (f Factory) WithGasPrices(gasPrices string) Factory {
	parsedGasPrices, err := sdk.ParseDecCoins(gasPrices)
	if err != nil {
		panic(err)
	}

	f.gasPrices = parsedGasPrices
	return f
}

// WithKeybase returns a copy of the Factory with updated Keybase.
func (f Factory) WithKeybase(keybase keyring.Keyring) Factory {
	f.keybase = keybase
	return f
}

// WithFromName returns a copy of the Factory with updated fromName
// fromName will be use for building a simulation tx.
func (f Factory) WithFromName(fromName string) Factory {
	f.fromName = fromName
	return f
}

// WithSequence returns a copy of the Factory with an updated sequence number.
func (f Factory) WithSequence(sequence uint64) Factory {
	f.sequence = sequence
	return f
}

// WithMemo returns a copy of the Factory with an updated memo.
func (f Factory) WithMemo(memo string) Factory {
	f.memo = memo
	return f
}

// WithAccountNumber returns a copy of the Factory with an updated account number.
func (f Factory) WithAccountNumber(accnum uint64) Factory {
	f.accountNumber = accnum
	return f
}

// WithGasAdjustment returns a copy of the Factory with an updated gas adjustment.
func (f Factory) WithGasAdjustment(gasAdj float64) Factory {
	f.gasAdjustment = gasAdj
	return f
}

// WithSimulateAndExecute returns a copy of the Factory with an updated gas
// simulation value.
func (f Factory) WithSimulateAndExecute(sim bool) Factory {
	f.simulateAndExecute = sim
	return f
}

// SignMode returns the sign mode configured in the Factory
func (f Factory) SignMode() signing.SignMode {
	return f.signMode
}

// WithSignMode returns a copy of the Factory with an updated sign mode value.
func (f Factory) WithSignMode(mode signing.SignMode) Factory {
	f.signMode = mode
	return f
}

// WithTimeoutHeight returns a copy of the Factory with an updated timeout height.
func (f Factory) WithTimeoutHeight(height uint64) Factory {
	f.timeoutHeight = height
	return f
}

// WithFeeGranter returns a copy of the Factory with an updated fee granter.
func (f Factory) WithFeeGranter(fg sdk.AccAddress) Factory {
	f.feeGranter = fg
	return f
}

// WithFeePayer returns a copy of the Factory with an updated fee granter.
func (f Factory) WithFeePayer(fp sdk.AccAddress) Factory {
	f.feePayer = fp
	return f
}

// WithPreprocessTxHook returns a copy of the Factory with an updated preprocess tx function,
// allows for preprocessing of transaction data using the TxBuilder.
func (f Factory) WithPreprocessTxHook(preprocessFn client.PreprocessTxFn) Factory {
	f.preprocessTxHook = preprocessFn
	return f
}

// PreprocessTx calls the preprocessing hook with the factory parameters and
// returns the result.
func (f Factory) PreprocessTx(keyname string, builder client.TxBuilder) error {
	if f.preprocessTxHook == nil {
		// Allow pass-through
		return nil
	}

	key, err := f.Keybase().Key(keyname)
	if err != nil {
		return fmt.Errorf("error retrieving key from keyring: %w", err)
	}

	return f.preprocessTxHook(f.chainID, key.GetType(), builder)
}

// WithExtensionOptions returns a Factory with given extension options added to the existing options,
// Example to add dynamic fee extension options:
//
//	extOpt := ethermint.ExtensionOptionDynamicFeeTx{
//		MaxPriorityPrice: math.NewInt(1000000),
//	}
//
//	extBytes, _ := extOpt.Marshal()
//
//	extOpts := []*types.Any{
//		{
//			TypeUrl: "/ethermint.types.v1.ExtensionOptionDynamicFeeTx",
//			Value:   extBytes,
//		},
//	}
//
// txf.WithExtensionOptions(extOpts...)
func (f Factory) WithExtensionOptions(extOpts ...*codectypes.Any) Factory {
	f.extOptions = extOpts
	return f
}

// BuildUnsignedTx builds a transaction to be signed given a set of messages.
// Once created, the fee, memo, and messages are set.
func (f Factory) BuildUnsignedTx(msgs ...sdk.Msg) (client.TxBuilder, error) {
	if f.offline && f.generateOnly {
		if f.chainID != "" {
			return nil, errors.New("chain ID cannot be used when offline and generate-only flags are set")
		}
	} else if f.chainID == "" {
		return nil, errors.New("chain ID required but not specified")
	}

	fees := f.fees

	if !f.gasPrices.IsZero() {
		if !fees.IsZero() {
			return nil, errors.New("cannot provide both fees and gas prices")
		}

		glDec := math.LegacyNewDec(int64(f.gas))

		// Derive the fees based on the provided gas prices, where
		// fee = ceil(gasPrice * gasLimit).
		fees = make(sdk.Coins, len(f.gasPrices))

		for i, gp := range f.gasPrices {
			fee := gp.Amount.Mul(glDec)
			fees[i] = sdk.NewCoin(gp.Denom, fee.Ceil().RoundInt())
		}
	}

	// Prevent simple inclusion of a valid mnemonic in the memo field
	if f.memo != "" && bip39.IsMnemonicValid(strings.ToLower(f.memo)) {
		return nil, errors.New("cannot provide a valid mnemonic seed in the memo field")
	}

	tx := f.txConfig.NewTxBuilder()

	if err := tx.SetMsgs(msgs...); err != nil {
		return nil, err
	}

	tx.SetMemo(f.memo)
	tx.SetFeeAmount(fees)
	tx.SetGasLimit(f.gas)
	tx.SetFeeGranter(f.feeGranter)
	tx.SetFeePayer(f.feePayer)
	tx.SetTimeoutHeight(f.TimeoutHeight())

	if etx, ok := tx.(client.ExtendedTxBuilder); ok {
		etx.SetExtensionOptions(f.extOptions...)
	}

	return tx, nil
}

// PrintUnsignedTx will generate an unsigned transaction and print it to the writer
// specified by ctx.Output. If simulation was requested, the gas will be
// simulated and also printed to the same writer before the transaction is
// printed.
func (f Factory) PrintUnsignedTx(clientCtx client.Context, msgs ...sdk.Msg) error {
	if f.SimulateAndExecute() {
		if clientCtx.Offline {
			return errors.New("cannot estimate gas in offline mode")
		}

		// Prepare TxFactory with acc & seq numbers as CalculateGas requires
		// account and sequence numbers to be set
		preparedTxf, err := f.Prepare(clientCtx)
		if err != nil {
			return err
		}

		_, adjusted, err := CalculateGas(clientCtx, preparedTxf, msgs...)
		if err != nil {
			return err
		}

		f = f.WithGas(adjusted)
		_, _ = fmt.Fprintf(os.Stderr, "%s\n", GasEstimateResponse{GasEstimate: f.Gas()})
	}

	unsignedTx, err := f.BuildUnsignedTx(msgs...)
	if err != nil {
		return err
	}

	encoder := f.txConfig.TxJSONEncoder()
	if encoder == nil {
		return errors.New("cannot print unsigned tx: tx json encoder is nil")
	}

	json, err := encoder(unsignedTx.GetTx())
	if err != nil {
		return err
	}

	return clientCtx.PrintString(fmt.Sprintf("%s\n", json))
}

// BuildSimTx creates an unsigned tx with an empty single signature and returns
// the encoded transaction or an error if the unsigned transaction cannot be
// built.
func (f Factory) BuildSimTx(msgs ...sdk.Msg) ([]byte, error) {
	txb, err := f.BuildUnsignedTx(msgs...)
	if err != nil {
		return nil, err
	}

	pk, err := f.getSimPK()
	if err != nil {
		return nil, err
	}

	// Create an empty signature literal as the ante handler will populate with a
	// sentinel pubkey.
	sig := signing.SignatureV2{
		PubKey:   pk,
		Data:     f.getSimSignatureData(pk),
		Sequence: f.Sequence(),
	}
	if err := txb.SetSignatures(sig); err != nil {
		return nil, err
	}

	encoder := f.txConfig.TxEncoder()
	if encoder == nil {
		return nil, fmt.Errorf("cannot simulate tx: tx encoder is nil")
	}

	return encoder(txb.GetTx())
}

// getSimPK gets the public key to use for building a simulation tx.
// Note, we should only check for keys in the keybase if we are in simulate and execute mode,
// e.g. when using --gas=auto.
// When using --dry-run, we are is simulation mode only and should not check the keybase.
// Ref: https://github.com/cosmos/cosmos-sdk/issues/11283
func (f Factory) getSimPK() (cryptotypes.PubKey, error) {
	var (
		ok bool
		pk cryptotypes.PubKey = &secp256k1.PubKey{} // use default public key type
	)

	if f.simulateAndExecute && f.keybase != nil {
		record, err := f.keybase.Key(f.fromName)
		if err != nil {
			return nil, err
		}

		pk, ok = record.PubKey.GetCachedValue().(cryptotypes.PubKey)
		if !ok {
			return nil, errors.New("cannot build signature for simulation, failed to convert proto Any to public key")
		}
	}

	return pk, nil
}

// getSimSignatureData based on the pubKey type gets the correct SignatureData type
// to use for building a simulation tx.
func (f Factory) getSimSignatureData(pk cryptotypes.PubKey) signing.SignatureData {
	multisigPubKey, ok := pk.(*multisig.LegacyAminoPubKey)
	if !ok {
		return &signing.SingleSignatureData{SignMode: f.signMode}
	}

	multiSignatureData := make([]signing.SignatureData, 0, multisigPubKey.Threshold)
	for i := uint32(0); i < multisigPubKey.Threshold; i++ {
		multiSignatureData = append(multiSignatureData, &signing.SingleSignatureData{
			SignMode: f.SignMode(),
		})
	}

	return &signing.MultiSignatureData{
		Signatures: multiSignatureData,
	}
}

// Prepare ensures the account defined by ctx.GetFromAddress() exists and
// if the account number and/or the account sequence number are zero (not set),
// they will be queried for and set on the provided Factory.
// A new Factory with the updated fields will be returned.
// Note: When in offline mode, the Prepare does nothing and returns the original factory.
func (f Factory) Prepare(clientCtx client.Context) (Factory, error) {
	if clientCtx.Offline {
		return f, nil
	}

	fc := f
	from := clientCtx.GetFromAddress()

	if err := fc.accountRetriever.EnsureExists(clientCtx, from); err != nil {
		return fc, err
	}

	initNum, initSeq := fc.accountNumber, fc.sequence
	if initNum == 0 || initSeq == 0 {
		num, seq, err := fc.accountRetriever.GetAccountNumberSequence(clientCtx, from)
		if err != nil {
			return fc, err
		}

		if initNum == 0 {
			fc = fc.WithAccountNumber(num)
		}

		if initSeq == 0 {
			fc = fc.WithSequence(seq)
		}
	}

	return fc, nil
}

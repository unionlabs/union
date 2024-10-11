package keys

import (
	"errors"
	"fmt"

	"github.com/mdp/qrterminal/v3"
	"github.com/spf13/cobra"

	"cosmossdk.io/core/address"
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/ledger"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerr "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	// FlagAddress is the flag for the user's address on the command line.
	FlagAddress = "address"
	// FlagPublicKey represents the user's public key on the command line.
	FlagPublicKey = "pubkey"
	// FlagBechPrefix defines a desired Bech32 prefix encoding for a key.
	FlagBechPrefix = "bech"
	// FlagDevice indicates that the information should be shown in the device
	FlagDevice = "device"

	flagMultiSigThreshold = "multisig-threshold"
	flagQRCode            = "qrcode"
)

// ShowKeysCmd shows key information for a given key name.
func ShowKeysCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "show <name_or_address> [name_or_address...]",
		Short: "Retrieve key information by name or address",
		Long: `Display keys details. If multiple names or addresses are provided,
then an ephemeral multisig key will be created under the name "multi"
consisting of all the keys provided by name and multisig threshold.`,
		Args: cobra.MinimumNArgs(1),
		RunE: runShowCmd,
	}
	f := cmd.Flags()
	f.String(FlagBechPrefix, sdk.PrefixAccount, "The Bech32 prefix encoding for a key (acc|val|cons)")
	f.BoolP(FlagAddress, "a", false, "Output the address only (cannot be used with --output)")
	f.BoolP(FlagPublicKey, "p", false, "Output the public key only (cannot be used with --output)")
	f.BoolP(FlagDevice, "d", false, "Output the address in a ledger device (cannot be used with --pubkey)")
	f.Int(flagMultiSigThreshold, 1, "K out of N required signatures")
	f.Bool(flagQRCode, false, "Display key address QR code (will be ignored if -a or --address is false)")

	return cmd
}

func runShowCmd(cmd *cobra.Command, args []string) (err error) {
	k := new(keyring.Record)
	clientCtx, err := client.GetClientQueryContext(cmd)
	if err != nil {
		return err
	}
	outputFormat := clientCtx.OutputFormat

	if len(args) == 1 {
		k, err = fetchKey(clientCtx.Keyring, args[0], clientCtx.AddressCodec)
		if err != nil {
			return fmt.Errorf("%s is not a valid name or address: %w", args[0], err)
		}
	} else {
		pks := make([]cryptotypes.PubKey, len(args))
		seenKeys := make(map[string]struct{})
		for i, keyRef := range args {
			if _, ok := seenKeys[keyRef]; ok {
				// we just show warning message instead of return error in case someone relies on this behavior.
				cmd.PrintErrf("WARNING: duplicate keys found: %s.\n\n", keyRef)
			} else {
				seenKeys[keyRef] = struct{}{}
			}

			k, err := fetchKey(clientCtx.Keyring, keyRef, clientCtx.AddressCodec)
			if err != nil {
				return fmt.Errorf("%s is not a valid name or address: %w", keyRef, err)
			}

			key, err := k.GetPubKey()
			if err != nil {
				return err
			}
			pks[i] = key
		}

		multisigThreshold, _ := cmd.Flags().GetInt(flagMultiSigThreshold)

		if err := validateMultisigThreshold(multisigThreshold, len(args)); err != nil {
			return err
		}

		multikey := multisig.NewLegacyAminoPubKey(multisigThreshold, pks)
		k, err = keyring.NewMultiRecord(k.Name, multikey)
		if err != nil {
			return err
		}
	}

	isShowAddr, _ := cmd.Flags().GetBool(FlagAddress)
	isShowPubKey, _ := cmd.Flags().GetBool(FlagPublicKey)
	isShowDevice, _ := cmd.Flags().GetBool(FlagDevice)
	isShowQRCode, _ := cmd.Flags().GetBool(flagQRCode)

	isOutputSet := false
	tmp := cmd.Flag(flags.FlagOutput)
	if tmp != nil {
		isOutputSet = tmp.Changed
	}

	if isShowAddr && isShowPubKey {
		return errors.New("cannot use both --address and --pubkey at once")
	}

	if isOutputSet && (isShowAddr || isShowPubKey) {
		return errors.New("cannot use --output with --address or --pubkey")
	}

	bechPrefix, _ := cmd.Flags().GetString(FlagBechPrefix)
	ko, err := getKeyOutput(clientCtx, bechPrefix, k)
	if err != nil {
		return err
	}

	if isOutputSet {
		clientCtx.OutputFormat, _ = cmd.Flags().GetString(flags.FlagOutput)
	}

	switch {
	case isShowAddr, isShowPubKey:
		out := ko.Address
		if isShowPubKey {
			out = ko.PubKey
		} else if isShowQRCode {
			qrterminal.GenerateHalfBlock(out, qrterminal.H, cmd.OutOrStdout())
		}

		if _, err := fmt.Fprintln(cmd.OutOrStdout(), out); err != nil {
			return err
		}
	default:
		if err := printKeyringRecord(cmd.OutOrStdout(), ko, outputFormat); err != nil {
			return err
		}
	}

	if isShowDevice {
		if isShowPubKey {
			return errors.New("the device flag (-d) can only be used for addresses not pubkeys")
		}
		if bechPrefix != "acc" {
			return errors.New("the device flag (-d) can only be used for accounts")
		}

		// Override and show in the device
		if k.GetType() != keyring.TypeLedger {
			return errors.New("the device flag (-d) can only be used for accounts stored in devices")
		}

		ledgerItem := k.GetLedger()
		if ledgerItem == nil {
			return errors.New("unable to get ledger item")
		}

		pk, err := k.GetPubKey()
		if err != nil {
			return err
		}

		return ledger.ShowAddress(*ledgerItem.Path, pk, clientCtx.AddressPrefix)
	}

	return nil
}

func fetchKey(kb keyring.Keyring, keyref string, addressCodec address.Codec) (*keyring.Record, error) {
	// firstly check if the keyref is a key name of a key registered in a keyring.
	k, err := kb.Key(keyref)
	// if the key is not there or if we have a problem with a keyring itself then we move to a
	// fallback: searching for key by address.

	if err == nil || !errorsmod.IsOf(err, sdkerr.ErrIO, sdkerr.ErrKeyNotFound) {
		return k, err
	}

	accAddr, err := addressCodec.StringToBytes(keyref)
	if err != nil {
		return k, err
	}

	k, err = kb.KeyByAddress(accAddr)
	return k, errorsmod.Wrap(err, "Invalid key")
}

func validateMultisigThreshold(k, nKeys int) error {
	if k <= 0 {
		return errors.New("threshold must be a positive integer")
	}
	if nKeys < k {
		return fmt.Errorf(
			"threshold k of n multisignature: %d < %d", nKeys, k)
	}
	return nil
}

func getKeyOutput(clientCtx client.Context, bechPrefix string, k *keyring.Record) (KeyOutput, error) {
	switch bechPrefix {
	case sdk.PrefixAccount:
		return MkAccKeyOutput(k, clientCtx.AddressCodec)
	case sdk.PrefixValidator:
		return MkValKeyOutput(k, clientCtx.ValidatorAddressCodec)
	case sdk.PrefixConsensus:
		return MkConsKeyOutput(k, clientCtx.ConsensusAddressCodec)
	}

	return KeyOutput{}, fmt.Errorf("invalid Bech32 prefix encoding provided: %s", bechPrefix)
}

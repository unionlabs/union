package keys

import (
	"bufio"
	"bytes"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"sort"
	"strings"

	"github.com/cosmos/go-bip39"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/input"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	flagInteractive  = "interactive"
	flagRecover      = "recover"
	flagNoBackup     = "no-backup"
	flagCoinType     = "coin-type"
	flagAccount      = "account"
	flagIndex        = "index"
	flagMultisig     = "multisig"
	flagNoSort       = "nosort"
	flagHDPath       = "hd-path"
	flagPubKeyBase64 = "pubkey-base64"
	flagIndiscreet   = "indiscreet"
	flagMnemonicSrc  = "source"

	// DefaultKeyPass contains the default key password for genesis transactions
	DefaultKeyPass = "12345678"
)

// AddKeyCommand defines a keys command to add a generated or recovered private key to keybase.
func AddKeyCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "add <name>",
		Short: "Add an encrypted private key (either newly generated or recovered), encrypt it, and save to <name> file",
		Long: `Derive a new private key and encrypt to disk.
Optionally specify a BIP39 mnemonic, a BIP39 passphrase to further secure the mnemonic,
and a bip32 HD path to derive a specific account. The key will be stored under the given name
and encrypted with the given password. The only input that is required is the encryption password.

If run with -i, it will prompt the user for BIP44 path, BIP39 mnemonic, and passphrase.
The flag --recover allows one to recover a key from a seed passphrase.
If run with --dry-run, a key would be generated (or recovered) but not stored to the
local keystore.
Use the --pubkey flag to add arbitrary public keys to the keystore for constructing
multisig transactions.

Use the --source flag to import mnemonic from a file in recover or interactive mode. 
Example:

	keys add testing --recover --source ./mnemonic.txt

You can create and store a multisig key by passing the list of key names stored in a keyring
and the minimum number of signatures required through --multisig-threshold. The keys are
sorted by address, unless the flag --nosort is set.
Example:

    keys add mymultisig --multisig "keyname1,keyname2,keyname3" --multisig-threshold 2
`,
		Args: cobra.ExactArgs(1),
		RunE: runAddCmdPrepare,
	}
	f := cmd.Flags()
	f.StringSlice(flagMultisig, nil, "List of key names stored in keyring to construct a public legacy multisig key")
	f.Int(flagMultiSigThreshold, 1, "K out of N required signatures. For use in conjunction with --multisig")
	f.Bool(flagNoSort, false, "Keys passed to --multisig are taken in the order they're supplied")
	f.String(FlagPublicKey, "", "Parse a public key in JSON format and saves key info to <name> file.")
	f.String(flagPubKeyBase64, "", "Parse a public key in base64 format and saves key info.")
	f.BoolP(flagInteractive, "i", false, "Interactively prompt user for BIP39 passphrase and mnemonic")
	f.Bool(flags.FlagUseLedger, false, "Store a local reference to a private key on a Ledger device")
	f.Bool(flagRecover, false, "Provide seed phrase to recover existing key instead of creating")
	f.Bool(flagNoBackup, false, "Don't print out seed phrase (if others are watching the terminal)")
	f.Bool(flags.FlagDryRun, false, "Perform action, but don't add key to local keystore")
	f.String(flagHDPath, "", "Manual HD Path derivation (overrides BIP44 config)")
	f.Uint32(flagCoinType, sdk.CoinType, "coin type number for HD derivation")
	f.Uint32(flagAccount, 0, "Account number for HD derivation (less than equal 2147483647)")
	f.Uint32(flagIndex, 0, "Address index number for HD derivation (less than equal 2147483647)")
	f.String(flags.FlagKeyType, string(hd.Secp256k1Type), "Key signing algorithm to generate keys for")
	f.Bool(flagIndiscreet, false, "Print seed phrase directly on current terminal (only valid when --no-backup is false)")
	f.String(flagMnemonicSrc, "", "Import mnemonic from a file (only usable when recover or interactive is passed)")

	// support old flags name for backwards compatibility
	f.SetNormalizeFunc(func(f *pflag.FlagSet, name string) pflag.NormalizedName {
		if name == flags.FlagKeyAlgorithm {
			name = flags.FlagKeyType
		}

		return pflag.NormalizedName(name)
	})

	return cmd
}

func runAddCmdPrepare(cmd *cobra.Command, args []string) error {
	clientCtx, err := client.GetClientQueryContext(cmd)
	if err != nil {
		return err
	}

	buf := bufio.NewReader(clientCtx.Input)
	return runAddCmd(clientCtx, cmd, args, buf)
}

/*
input
  - bip39 mnemonic
  - bip39 passphrase
  - bip44 path
  - local encryption password

output
  - armor encrypted private key (saved to file)
*/
func runAddCmd(ctx client.Context, cmd *cobra.Command, args []string, inBuf *bufio.Reader) error {
	var err error

	name := args[0]
	if strings.TrimSpace(name) == "" {
		return errors.New("the provided name is invalid or empty after trimming whitespace")
	}
	interactive, _ := cmd.Flags().GetBool(flagInteractive)
	kb := ctx.Keyring
	outputFormat := ctx.OutputFormat

	keyringAlgos, _ := kb.SupportedAlgorithms()
	algoStr, _ := cmd.Flags().GetString(flags.FlagKeyType)
	algo, err := keyring.NewSigningAlgoFromString(algoStr, keyringAlgos)
	if err != nil {
		return err
	}

	if dryRun, _ := cmd.Flags().GetBool(flags.FlagDryRun); dryRun {
		// use in memory keybase
		kb = keyring.NewInMemory(ctx.Codec)
	} else {
		_, err = kb.Key(name)
		if err == nil {
			// account exists, ask for user confirmation
			response, err2 := input.GetConfirmation(fmt.Sprintf("override the existing name %s", name), inBuf, cmd.ErrOrStderr())
			if err2 != nil {
				return err2
			}

			if !response {
				return errors.New("aborted")
			}

			err2 = kb.Delete(name)
			if err2 != nil {
				return err2
			}
		}

		multisigKeys, _ := cmd.Flags().GetStringSlice(flagMultisig)
		if len(multisigKeys) != 0 {
			pks := make([]cryptotypes.PubKey, len(multisigKeys))
			multisigThreshold, _ := cmd.Flags().GetInt(flagMultiSigThreshold)
			if err := validateMultisigThreshold(multisigThreshold, len(multisigKeys)); err != nil {
				return err
			}

			seenKeys := make(map[string]struct{})
			for i, keyName := range multisigKeys {
				if _, ok := seenKeys[keyName]; ok {
					return fmt.Errorf("duplicate multisig keys: %s", keyName)
				}
				seenKeys[keyName] = struct{}{}

				k, err := kb.Key(keyName)
				if err != nil {
					return err
				}

				key, err := k.GetPubKey()
				if err != nil {
					return err
				}
				pks[i] = key
			}

			if noSort, _ := cmd.Flags().GetBool(flagNoSort); !noSort {
				sort.Slice(pks, func(i, j int) bool {
					return bytes.Compare(pks[i].Address(), pks[j].Address()) < 0
				})
			}

			pk := multisig.NewLegacyAminoPubKey(multisigThreshold, pks)
			k, err := kb.SaveMultisig(name, pk)
			if err != nil {
				return err
			}

			return printCreate(ctx, cmd, k, false, false, "", outputFormat)
		}
	}

	pubKey, _ := cmd.Flags().GetString(FlagPublicKey)
	pubKeyBase64, _ := cmd.Flags().GetString(flagPubKeyBase64)
	if pubKey != "" && pubKeyBase64 != "" {
		return fmt.Errorf(`flags %s and %s cannot be used simultaneously`, FlagPublicKey, flagPubKeyBase64)
	}
	if pubKey != "" {
		var pk cryptotypes.PubKey
		if err = ctx.Codec.UnmarshalInterfaceJSON([]byte(pubKey), &pk); err != nil {
			return err
		}

		k, err := kb.SaveOfflineKey(name, pk)
		if err != nil {
			return err
		}

		return printCreate(ctx, cmd, k, false, false, "", outputFormat)
	}
	if pubKeyBase64 != "" {
		b64, err := base64.StdEncoding.DecodeString(pubKeyBase64)
		if err != nil {
			return err
		}

		var pk cryptotypes.PubKey

		// create an empty seckp256k1 pubkey since it is the key returned by algo Generate function.
		enotySecpPubKey, err := codectypes.NewAnyWithValue(&secp256k1.PubKey{})
		if err != nil {
			return err
		}

		jsonPub, err := json.Marshal(struct {
			Type string `json:"@type,omitempty"`
			Key  string `json:"key,omitempty"`
		}{enotySecpPubKey.TypeUrl, string(b64)})
		if err != nil {
			return fmt.Errorf("failed to JSON marshal typeURL and base64 key: %w", err)
		}

		if err = ctx.Codec.UnmarshalInterfaceJSON(jsonPub, &pk); err != nil {
			return err
		}

		k, err := kb.SaveOfflineKey(name, pk)
		if err != nil {
			return fmt.Errorf("failed to save offline key: %w", err)
		}

		return printCreate(ctx, cmd, k, false, false, "", outputFormat)
	}

	coinType, _ := cmd.Flags().GetUint32(flagCoinType)
	account, _ := cmd.Flags().GetUint32(flagAccount)
	index, _ := cmd.Flags().GetUint32(flagIndex)
	hdPath, _ := cmd.Flags().GetString(flagHDPath)
	useLedger, _ := cmd.Flags().GetBool(flags.FlagUseLedger)

	if len(hdPath) == 0 {
		hdPath = hd.CreateHDPath(coinType, account, index).String()
	} else if useLedger {
		return errors.New("cannot set custom bip32 path with ledger")
	}

	// If we're using ledger, only thing we need is the path and the bech32 prefix.
	if useLedger {
		bech32PrefixAccAddr := ctx.AddressPrefix
		k, err := kb.SaveLedgerKey(name, hd.Secp256k1, bech32PrefixAccAddr, coinType, account, index)
		if err != nil {
			return err
		}

		return printCreate(ctx, cmd, k, false, false, "", outputFormat)
	}

	// Get bip39 mnemonic
	var mnemonic, bip39Passphrase string

	recoverFlag, _ := cmd.Flags().GetBool(flagRecover)
	mnemonicSrc, _ := cmd.Flags().GetString(flagMnemonicSrc)
	if recoverFlag {
		if mnemonicSrc != "" {
			mnemonic, err = readMnemonicFromFile(mnemonicSrc)
			if err != nil {
				return err
			}
		} else {
			mnemonic, err = input.GetString("Enter your bip39 mnemonic", inBuf)
			if err != nil {
				return err
			}
		}

		if !bip39.IsMnemonicValid(mnemonic) {
			return errors.New("invalid mnemonic")
		}
	} else if interactive {
		if mnemonicSrc != "" {
			mnemonic, err = readMnemonicFromFile(mnemonicSrc)
			if err != nil {
				return err
			}
		} else {
			mnemonic, err = input.GetString("Enter your bip39 mnemonic, or hit enter to generate one.", inBuf)
			if err != nil {
				return err
			}
		}

		if !bip39.IsMnemonicValid(mnemonic) && mnemonic != "" {
			return errors.New("invalid mnemonic")
		}
	}

	if len(mnemonic) == 0 {
		// read entropy seed straight from cmtcrypto.Rand and convert to mnemonic
		entropySeed, err := bip39.NewEntropy(mnemonicEntropySize)
		if err != nil {
			return err
		}

		mnemonic, err = bip39.NewMnemonic(entropySeed)
		if err != nil {
			return err
		}
	}

	// override bip39 passphrase
	if interactive {
		bip39Passphrase, err = input.GetSecretString(
			"Enter your bip39 passphrase. This is combined with the mnemonic to derive the seed. "+
				"Most users should just hit enter to use the default, \"\"\n", inBuf)
		if err != nil {
			return err
		}

		// if they use one, make them re-enter it
		if len(bip39Passphrase) != 0 {
			p2, err := input.GetSecretString("Repeat the passphrase:\n", inBuf)
			if err != nil {
				return err
			}

			if bip39Passphrase != p2 {
				return errors.New("passphrases don't match")
			}
		}
	}

	k, err := kb.NewAccount(name, mnemonic, bip39Passphrase, hdPath, algo)
	if err != nil {
		return err
	}
	noBackup, _ := cmd.Flags().GetBool(flagNoBackup)
	showMnemonic := !noBackup
	showMnemonicIndiscreetly, _ := cmd.Flags().GetBool(flagIndiscreet)

	// Recover key from seed passphrase
	if recoverFlag {
		// Hide mnemonic from output
		showMnemonic = false
		showMnemonicIndiscreetly = false
		mnemonic = ""
	}

	return printCreate(ctx, cmd, k, showMnemonic, showMnemonicIndiscreetly, mnemonic, outputFormat)
}

func printCreate(ctx client.Context, cmd *cobra.Command, k *keyring.Record, showMnemonic, showMnemonicIndiscreetly bool, mnemonic, outputFormat string) error {
	switch outputFormat {
	case flags.OutputFormatText:
		cmd.PrintErrln()
		ko, err := MkAccKeyOutput(k, ctx.AddressCodec)
		if err != nil {
			return err
		}

		if err := printKeyringRecord(cmd.OutOrStdout(), ko, outputFormat); err != nil {
			return err
		}

		// print mnemonic unless requested not to.
		if showMnemonic {
			if showMnemonicIndiscreetly {
				if _, err = fmt.Fprintf(cmd.ErrOrStderr(), "\n**Important** write this mnemonic phrase in a safe place.\nIt is the only way to recover your account if you ever forget your password.\n\n%s\n", mnemonic); err != nil {
					return fmt.Errorf("failed to print mnemonic: %w", err)
				}
			} else {
				if err = printDiscreetly(ctx, cmd.ErrOrStderr(), "**Important** write this mnemonic phrase in a safe place.\nIt is the only way to recover your account if you ever forget your password.", mnemonic); err != nil {
					return fmt.Errorf("failed to print mnemonic: %w", err)
				}
			}
		}
	case flags.OutputFormatJSON:
		out, err := MkAccKeyOutput(k, ctx.AddressCodec)
		if err != nil {
			return err
		}

		if showMnemonic {
			out.Mnemonic = mnemonic
		}

		jsonString, err := json.Marshal(out)
		if err != nil {
			return err
		}

		cmd.Println(string(jsonString))

	default:
		return fmt.Errorf("invalid output format %s", outputFormat)
	}

	return nil
}

func readMnemonicFromFile(filePath string) (string, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return "", err
	}
	defer file.Close()

	bz, err := io.ReadAll(file)
	if err != nil {
		return "", err
	}
	return string(bz), nil
}

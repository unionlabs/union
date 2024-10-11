package debug

import (
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"slices"
	"strconv"
	"strings"

	"github.com/spf13/cobra"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types/bech32/legacybech32" //nolint:staticcheck // we do old keys, they're keys after all.
	"github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/version"
)

var (
	flagPubkeyType = "type"
	ed             = "ed25519"
)

// Cmd creates a main CLI command
func Cmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "debug",
		Short: "Tool for helping with debugging your application",
		RunE:  client.ValidateCmd,
	}

	cmd.AddCommand(CodecCmd())
	cmd.AddCommand(PubkeyCmd())
	cmd.AddCommand(PubkeyRawCmd())
	cmd.AddCommand(AddrCmd())
	cmd.AddCommand(RawBytesCmd())
	cmd.AddCommand(PrefixesCmd())

	return cmd
}

// CodecCmd creates and returns a new codec debug cmd.
func CodecCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "codec",
		Short: "Tool for helping with debugging your application codec",
		RunE:  client.ValidateCmd,
	}

	cmd.AddCommand(getCodecInterfaces())
	cmd.AddCommand(getCodecInterfaceImpls())

	return cmd
}

// getCodecInterfaces creates and returns a new cmd used for listing all registered interfaces on the application codec.
func getCodecInterfaces() *cobra.Command {
	return &cobra.Command{
		Use:     "list-interfaces",
		Short:   "List all registered interface type URLs",
		Long:    "List all registered interface type URLs using the application codec",
		Example: fmt.Sprintf("%s debug codec list-interfaces", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)
			iFaces := clientCtx.Codec.InterfaceRegistry().ListAllInterfaces()

			slices.Sort(iFaces)
			for _, iFace := range iFaces {
				cmd.Println(iFace)
			}
			return nil
		},
	}
}

// getCodecInterfaceImpls creates and returns a new cmd used for listing all registered implementations of a given interface on the application codec.
func getCodecInterfaceImpls() *cobra.Command {
	return &cobra.Command{
		Use:     "list-implementations <interface>",
		Short:   "List the registered type URLs for the provided interface",
		Long:    "List the registered type URLs that can be used for the provided interface name using the application codec",
		Example: fmt.Sprintf("%s debug codec list-implementations cosmos.crypto.PubKey", version.AppName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)
			impls := clientCtx.Codec.InterfaceRegistry().ListImplementations(args[0])

			slices.Sort(impls)
			for _, imp := range impls {
				cmd.Println(imp)
			}
			return nil
		},
	}
}

// getPubKeyFromString decodes SDK PubKey using JSON marshaler.
func getPubKeyFromString(ctx client.Context, pkstr string) (cryptotypes.PubKey, error) {
	var pk cryptotypes.PubKey
	err := ctx.Codec.UnmarshalInterfaceJSON([]byte(pkstr), &pk)
	return pk, err
}

func PubkeyCmd() *cobra.Command {
	return &cobra.Command{
		Use:     "pubkey <pubkey>",
		Short:   "Decode a pubkey from proto JSON",
		Long:    "Decode a pubkey from proto JSON and display it's address.",
		Example: fmt.Sprintf(`%s debug pubkey '{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"AurroA7jvfPd1AadmmOvWM2rJSwipXfRf8yD6pLbA2DJ"}'`, version.AppName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)
			pk, err := getPubKeyFromString(clientCtx, args[0])
			if err != nil {
				return err
			}
			cmd.Println("Address:", pk.Address())
			cmd.Println("PubKey Hex:", hex.EncodeToString(pk.Bytes()))
			return nil
		},
	}
}

func bytesToPubkey(bz []byte, keytype string) (cryptotypes.PubKey, bool) {
	if keytype == ed {
		if len(bz) == ed25519.PubKeySize {
			return &ed25519.PubKey{Key: bz}, true
		}
	}

	if len(bz) == secp256k1.PubKeySize {
		return &secp256k1.PubKey{Key: bz}, true
	}
	return nil, false
}

// getPubKeyFromRawString returns a PubKey (PubKeyEd25519 or PubKeySecp256k1) by attempting
// to decode the pubkey string from hex, base64, and finally bech32. If all
// encodings fail, an error is returned.
func getPubKeyFromRawString(pkstr, keytype string) (cryptotypes.PubKey, error) {
	// Try hex decoding
	bz, err := hex.DecodeString(pkstr)
	if err == nil {
		pk, ok := bytesToPubkey(bz, keytype)
		if ok {
			return pk, nil
		}
	}

	bz, err = base64.StdEncoding.DecodeString(pkstr)
	if err == nil {
		pk, ok := bytesToPubkey(bz, keytype)
		if ok {
			return pk, nil
		}
	}

	pk, err := legacybech32.UnmarshalPubKey(legacybech32.AccPK, pkstr)
	if err == nil {
		return pk, nil
	}

	pk, err = legacybech32.UnmarshalPubKey(legacybech32.ValPK, pkstr)
	if err == nil {
		return pk, nil
	}

	pk, err = legacybech32.UnmarshalPubKey(legacybech32.ConsPK, pkstr)
	if err == nil {
		return pk, nil
	}

	return nil, fmt.Errorf("pubkey '%s' invalid; expected hex, base64, or bech32 of correct size", pkstr)
}

func PubkeyRawCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "pubkey-raw <pubkey> [-t {ed25519, secp256k1}]",
		Short: "Decode a ED25519 or secp256k1 pubkey from hex, base64, or bech32",
		Long:  "Decode a pubkey from hex, base64, or bech32.",
		Example: fmt.Sprintf(`
%s debug pubkey-raw 8FCA9D6D1F80947FD5E9A05309259746F5F72541121766D5F921339DD061174A
%s debug pubkey-raw j8qdbR+AlH/V6aBTCSWXRvX3JUESF2bV+SEzndBhF0o=
%s debug pubkey-raw cosmospub1zcjduepq3l9f6mglsz28l40f5pfsjfvhgm6lwf2pzgtkd40eyyeem5rpza9q47axrz
			`, version.AppName, version.AppName, version.AppName),
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)

			pubkeyType, err := cmd.Flags().GetString(flagPubkeyType)
			if err != nil {
				return err
			}
			pubkeyType = strings.ToLower(pubkeyType)
			if pubkeyType != "secp256k1" && pubkeyType != ed {
				return errorsmod.Wrapf(errors.ErrInvalidType, "invalid pubkey type, expected oneof ed25519 or secp256k1")
			}

			pk, err := getPubKeyFromRawString(args[0], pubkeyType)
			if err != nil {
				return err
			}

			var consensusPub string
			edPK, ok := pk.(*ed25519.PubKey)
			if ok && pubkeyType == ed {
				consensusPub, err = legacybech32.MarshalPubKey(legacybech32.ConsPK, edPK)
				if err != nil {
					return err
				}

				cmd.Printf("Hex: %X\n", edPK.Key)
			}
			cmd.Println("Parsed key as", pk.Type())

			pubKeyJSONBytes, err := clientCtx.LegacyAmino.MarshalJSON(pk)
			if err != nil {
				return err
			}
			accPub, err := legacybech32.MarshalPubKey(legacybech32.AccPK, pk)
			if err != nil {
				return err
			}
			valPub, err := legacybech32.MarshalPubKey(legacybech32.ValPK, pk)
			if err != nil {
				return err
			}
			cmd.Println("Address:", pk.Address())
			cmd.Println("JSON (base64):", string(pubKeyJSONBytes))
			cmd.Println("Bech32 Acc:", accPub)
			cmd.Println("Bech32 Validator Operator:", valPub)
			if pubkeyType == "ed25519" {
				cmd.Println("Bech32 Validator Consensus:", consensusPub)
			}

			return nil
		},
	}
	cmd.Flags().StringP(flagPubkeyType, "t", ed, "Pubkey type to decode (oneof secp256k1, ed25519)")
	return cmd
}

func AddrCmd() *cobra.Command {
	return &cobra.Command{
		Use:     "addr <address>",
		Short:   "Convert an address between hex and bech32",
		Example: fmt.Sprintf("%s debug addr cosmos1e0jnq2sun3dzjh8p2xq95kk0expwmd7shwjpfg", version.AppName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)

			addrString := args[0]
			// try hex, then bech32
			var (
				addr []byte
				err  error
			)
			decodeFns := []func(text string) ([]byte, error){
				hex.DecodeString,
				clientCtx.AddressCodec.StringToBytes,
				clientCtx.ValidatorAddressCodec.StringToBytes,
				clientCtx.ConsensusAddressCodec.StringToBytes,
			}
			errs := make([]any, 0, len(decodeFns))
			for _, fn := range decodeFns {
				if addr, err = fn(addrString); err == nil {
					break
				}
				errs = append(errs, err)
			}
			if len(errs) == len(decodeFns) {
				errTags := []string{
					"hex", "bech32 acc", "bech32 val", "bech32 con",
				}
				format := ""
				for i := range errs {
					if format != "" {
						format += ", "
					}
					format += errTags[i] + ": %w"
				}
				return fmt.Errorf("expected hex or bech32. Got errors: "+format, errs...)
			}

			acc, _ := clientCtx.AddressCodec.BytesToString(addr)
			val, _ := clientCtx.ValidatorAddressCodec.BytesToString(addr)
			con, _ := clientCtx.ConsensusAddressCodec.BytesToString(addr)

			cmd.Println("Address:", addr)
			cmd.Printf("Address (hex): %X\n", addr)
			cmd.Printf("Bech32 Acc: %s\n", acc)
			cmd.Printf("Bech32 Val: %s\n", val)
			cmd.Printf("Bech32 Con: %s\n", con)
			return nil
		},
	}
}

func RawBytesCmd() *cobra.Command {
	return &cobra.Command{
		Use:     "raw-bytes <raw-bytes>",
		Short:   "Convert raw bytes output (eg. [10 21 13 255]) to hex",
		Long:    "Convert raw-bytes to hex.",
		Example: fmt.Sprintf("%s debug raw-bytes '[72 101 108 108 111 44 32 112 108 97 121 103 114 111 117 110 100]'", version.AppName),
		Args:    cobra.ExactArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			stringBytes := args[0]
			stringBytes = strings.Trim(stringBytes, "[")
			stringBytes = strings.Trim(stringBytes, "]")
			spl := strings.Split(stringBytes, " ")

			byteArray := []byte{}
			for _, s := range spl {
				b, err := strconv.ParseInt(s, 10, 8)
				if err != nil {
					return err
				}
				byteArray = append(byteArray, byte(b))
			}
			fmt.Printf("%X\n", byteArray)
			return nil
		},
	}
}

func PrefixesCmd() *cobra.Command {
	return &cobra.Command{
		Use:     "prefixes",
		Short:   "List prefixes used for Human-Readable Part (HRP) in Bech32",
		Example: fmt.Sprintf("$ %s debug prefixes", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)

			acc, _ := clientCtx.AddressCodec.BytesToString([]byte{})
			val, _ := clientCtx.ValidatorAddressCodec.BytesToString([]byte{})
			cons, _ := clientCtx.ConsensusAddressCodec.BytesToString([]byte{})

			checksumLen := 7
			if _, ok := clientCtx.AddressCodec.(addresscodec.Bech32Codec); !ok {
				cmd.Printf("%s uses custom address codec, this command may not work as expected.\n", version.AppName)
				checksumLen = 0
			}

			cmd.Printf("Bech32 Acc: %s\n", acc[:len(acc)-checksumLen])
			cmd.Printf("Bech32 Val: %s\n", val[:len(val)-checksumLen])
			cmd.Printf("Bech32 Con: %s\n", cons[:len(cons)-checksumLen])

			return nil
		},
	}
}

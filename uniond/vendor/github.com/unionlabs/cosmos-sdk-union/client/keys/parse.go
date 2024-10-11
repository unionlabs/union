package keys

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strings"

	"github.com/spf13/cobra"
	"sigs.k8s.io/yaml"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/bech32"
)

func bech32Prefixes(mainBech32Prefix string) []string {
	return []string{
		mainBech32Prefix,
		sdk.GetBech32PrefixAccPub(mainBech32Prefix),
		sdk.GetBech32PrefixValAddr(mainBech32Prefix),
		sdk.GetBech32PrefixValPub(mainBech32Prefix),
		sdk.GetBech32PrefixConsAddr(mainBech32Prefix),
		sdk.GetBech32PrefixConsPub(mainBech32Prefix),
	}
}

type hexOutput struct {
	Human string `json:"human"`
	Bytes string `json:"bytes"`
}

func (ho hexOutput) String() string {
	return fmt.Sprintf("Human readable part: %v\nBytes (hex): %s", ho.Human, ho.Bytes)
}

func newHexOutput(human string, bs []byte) hexOutput {
	return hexOutput{Human: human, Bytes: fmt.Sprintf("%X", bs)}
}

type bech32Output struct {
	Formats []string `json:"formats"`
}

func newBech32Output(bech32Prefix string, bs []byte) bech32Output {
	bech32Prefixes := bech32Prefixes(bech32Prefix)
	out := bech32Output{Formats: make([]string, len(bech32Prefixes))}

	for i, prefix := range bech32Prefixes {
		bech32Addr, err := bech32.ConvertAndEncode(prefix, bs)
		if err != nil {
			panic(err)
		}

		out.Formats[i] = bech32Addr
	}

	return out
}

func (bo bech32Output) String() string {
	out := make([]string, len(bo.Formats))

	for i, format := range bo.Formats {
		out[i] = fmt.Sprintf("  - %s", format)
	}

	return fmt.Sprintf("Bech32 Formats:\n%s", strings.Join(out, "\n"))
}

// ParseKeyStringCommand parses an address from hex to bech32 and vice versa.
func ParseKeyStringCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "parse <hex-or-bech32-address>",
		Short: "Parse address from hex to bech32 and vice versa",
		Long: `Convert and print to stdout key addresses and fingerprints from
hexadecimal into bech32 cosmos prefixed format and vice versa.
`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			return doParseKey(cmd, clientCtx.AddressPrefix, args)
		},
	}

	return cmd
}

func doParseKey(cmd *cobra.Command, bech32Prefix string, args []string) error {
	addr := strings.TrimSpace(args[0])
	outstream := cmd.OutOrStdout()

	if len(addr) == 0 {
		return errors.New("couldn't parse empty input")
	}

	output, _ := cmd.Flags().GetString(flags.FlagOutput)
	if !(runFromBech32(outstream, addr, output) || runFromHex(bech32Prefix, outstream, addr, output)) {
		return errors.New("couldn't find valid bech32 nor hex data")
	}

	return nil
}

// print info from bech32
func runFromBech32(w io.Writer, bech32str, output string) bool {
	hrp, bz, err := bech32.DecodeAndConvert(bech32str)
	if err != nil {
		return false
	}

	displayParseKeyInfo(w, newHexOutput(hrp, bz), output)

	return true
}

// print info from hex
func runFromHex(bech32Prefix string, w io.Writer, hexstr, output string) bool {
	bz, err := hex.DecodeString(hexstr)
	if err != nil {
		return false
	}

	displayParseKeyInfo(w, newBech32Output(bech32Prefix, bz), output)

	return true
}

func displayParseKeyInfo(w io.Writer, stringer fmt.Stringer, output string) {
	var (
		err error
		out []byte
	)

	switch output {
	case flags.OutputFormatText:
		out, err = yaml.Marshal(&stringer)

	case flags.OutputFormatJSON:
		out, err = json.Marshal(&stringer)
	}

	if err != nil {
		panic(err)
	}

	_, _ = fmt.Fprintln(w, string(out))
}

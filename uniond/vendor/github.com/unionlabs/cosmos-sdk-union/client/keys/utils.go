package keys

import (
	"encoding/json"
	"fmt"
	"io"

	"github.com/muesli/termenv"
	"sigs.k8s.io/yaml"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	cryptokeyring "github.com/cosmos/cosmos-sdk/crypto/keyring"
)

func printKeyringRecord(w io.Writer, ko KeyOutput, output string) error {
	switch output {
	case flags.OutputFormatText:
		if err := printTextRecords(w, []KeyOutput{ko}); err != nil {
			return err
		}

	case flags.OutputFormatJSON:
		out, err := json.Marshal(ko)
		if err != nil {
			return err
		}

		if _, err := fmt.Fprintln(w, string(out)); err != nil {
			return err
		}
	}

	return nil
}

func printKeyringRecords(clientCtx client.Context, w io.Writer, records []*cryptokeyring.Record, output string) error {
	kos, err := MkAccKeysOutput(records, clientCtx.AddressCodec)
	if err != nil {
		return err
	}

	switch output {
	case flags.OutputFormatText:
		if err := printTextRecords(w, kos); err != nil {
			return err
		}

	case flags.OutputFormatJSON:
		out, err := json.Marshal(kos)
		if err != nil {
			return err
		}

		if _, err := fmt.Fprintf(w, "%s", out); err != nil {
			return err
		}
	}

	return nil
}

func printTextRecords(w io.Writer, kos []KeyOutput) error {
	out, err := yaml.Marshal(&kos)
	if err != nil {
		return err
	}

	if _, err := fmt.Fprintln(w, string(out)); err != nil {
		return err
	}

	return nil
}

// printDiscreetly Print a secret string to an alternate screen, so the string isn't printed to the terminal.
func printDiscreetly(clientCtx client.Context, w io.Writer, promptMsg, secretMsg string) error {
	output := termenv.NewOutput(w)
	output.AltScreen()
	defer output.ExitAltScreen()
	if _, err := fmt.Fprintf(output, "%s\n\n%s\n\nPress 'Enter' key to continue.", promptMsg, secretMsg); err != nil {
		return err
	}
	if _, err := fmt.Scanln(); err != nil {
		return err
	}
	return nil
}

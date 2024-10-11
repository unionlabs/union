package commands

import (
	"fmt"

	"github.com/spf13/cobra"

	"github.com/cometbft/cometbft/crypto/ed25519"
	kt "github.com/cometbft/cometbft/internal/keytypes"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/privval"
)

// GenValidatorCmd allows the generation of a keypair for a
// validator.
var GenValidatorCmd = &cobra.Command{
	Use:     "gen-validator",
	Aliases: []string{"gen_validator"},
	Short:   "Generate new validator keypair",
	RunE:    genValidator,
}

func init() {
	GenValidatorCmd.Flags().StringVarP(&keyType, "key-type", "k", ed25519.KeyType, fmt.Sprintf("private key type (one of %s)", kt.SupportedKeyTypesStr()))
}

func genValidator(*cobra.Command, []string) error {
	pv, err := privval.GenFilePV("", "", genPrivKeyFromFlag)
	if err != nil {
		return fmt.Errorf("cannot generate file pv: %w", err)
	}
	jsbz, err := cmtjson.Marshal(pv)
	if err != nil {
		panic(err)
	}
	fmt.Printf(`%v
`, string(jsbz))
	return nil
}

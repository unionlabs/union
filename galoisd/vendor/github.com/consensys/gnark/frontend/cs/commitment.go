package cs

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"os"
	"strings"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/logger"
)

func Bsb22CommitmentComputePlaceholder(mod *big.Int, _ []*big.Int, output []*big.Int) (err error) {
	if (len(os.Args) > 0 && (strings.HasSuffix(os.Args[0], ".test") || strings.HasSuffix(os.Args[0], ".test.exe"))) || debug.Debug {
		// usually we only run solver without prover during testing
		log := logger.Logger()
		log.Error().Msg("Augmented commitment hint not replaced. Proof will not be sound and verification will fail!")
		output[0], err = rand.Int(rand.Reader, mod)
		if output[0].Sign() == 0 {
			// a commit == 0 is unlikely; happens quite often in tests
			// with tinyfield
			output[0].SetUint64(1)
		}
		return
	}
	return fmt.Errorf("placeholder function: to be replaced by commitment computation")
}

func init() {
	solver.RegisterHint(Bsb22CommitmentComputePlaceholder)
}

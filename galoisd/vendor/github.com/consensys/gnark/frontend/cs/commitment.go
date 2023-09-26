package cs

import (
	"crypto/rand"
	"fmt"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/logger"
	"hash/fnv"
	"math/big"
	"os"
	"strconv"
	"strings"
	"sync"
)

func Bsb22CommitmentComputePlaceholder(mod *big.Int, _ []*big.Int, output []*big.Int) (err error) {
	if (len(os.Args) > 0 && (strings.HasSuffix(os.Args[0], ".test") || strings.HasSuffix(os.Args[0], ".test.exe"))) || debug.Debug {
		// usually we only run solver without prover during testing
		log := logger.Logger()
		log.Error().Msg("Augmented commitment hint not replaced. Proof will not be sound and verification will fail!")
		output[0], err = rand.Int(rand.Reader, mod)
		return
	}
	return fmt.Errorf("placeholder function: to be replaced by commitment computation")
}

var maxNbCommitments int
var m sync.Mutex

func RegisterBsb22CommitmentComputePlaceholder(index int) (hintId solver.HintID, err error) {

	hintName := "bsb22 commitment #" + strconv.Itoa(index)
	// mimic solver.GetHintID
	hf := fnv.New32a()
	if _, err = hf.Write([]byte(hintName)); err != nil {
		return
	}
	hintId = solver.HintID(hf.Sum32())

	m.Lock()
	if maxNbCommitments == index {
		maxNbCommitments++
		solver.RegisterNamedHint(Bsb22CommitmentComputePlaceholder, hintId)
	}
	m.Unlock()

	return
}
func init() {
	solver.RegisterHint(Bsb22CommitmentComputePlaceholder)
}

package test

import (
	"encoding/hex"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"

	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/solidity"
	"github.com/consensys/gnark/backend/witness"
)

// solidityVerification checks that the exported solidity contract can verify the proof
// and that the proof is valid.
// It uses gnark-solidity-checker see test.WithSolidity option.
func (assert *Assert) solidityVerification(b backend.ID, vk solidity.VerifyingKey,
	proof any,
	validPublicWitness witness.Witness,
	opts []solidity.ExportOption,
) {
	if !SolcCheck || len(validPublicWitness.Vector().(fr_bn254.Vector)) == 0 {
		return // nothing to check, will make solc fail.
	}
	assert.t.Helper()

	// set default options for CI when none are provided
	if len(opts) == 0 {
		opts = append(opts, solidity.WithPragmaVersion("^0.8.0")) // to avoid needing sync Solidity CI all the time
	}

	// make temp dir
	tmpDir, err := os.MkdirTemp("", "gnark-solidity-check*")
	assert.NoError(err)
	defer os.RemoveAll(tmpDir)

	// export solidity contract
	fSolidity, err := os.Create(filepath.Join(tmpDir, "gnark_verifier.sol"))
	assert.NoError(err)

	err = vk.ExportSolidity(fSolidity, opts...)
	assert.NoError(err)

	err = fSolidity.Close()
	assert.NoError(err)

	// generate assets
	// gnark-solidity-checker generate --dir tmpdir --solidity contract_g16.sol
	cmd := exec.Command("gnark-solidity-checker", "generate", "--dir", tmpDir, "--solidity", "gnark_verifier.sol")
	assert.t.Log("running ", cmd.String())
	out, err := cmd.CombinedOutput()
	assert.NoError(err, string(out))

	// len(vk.K) - 1 == len(publicWitness) + len(commitments)
	numOfCommitments := vk.NbPublicWitness() - len(validPublicWitness.Vector().(fr_bn254.Vector))

	checkerOpts := []string{"verify"}
	if b == backend.GROTH16 {
		checkerOpts = append(checkerOpts, "--groth16")
	} else if b == backend.PLONK {
		checkerOpts = append(checkerOpts, "--plonk")
	} else {
		panic("not implemented")
	}

	// proof to hex
	_proof, ok := proof.(interface{ MarshalSolidity() []byte })
	if !ok {
		panic("proof does not implement MarshalSolidity()")
	}

	proofStr := hex.EncodeToString(_proof.MarshalSolidity())

	if numOfCommitments > 0 {
		checkerOpts = append(checkerOpts, "--commitment", strconv.Itoa(numOfCommitments))
	}

	// public witness to hex
	bPublicWitness, err := validPublicWitness.MarshalBinary()
	assert.NoError(err)
	// that's quite dirty...
	// first 4 bytes -> nbPublic
	// next 4 bytes -> nbSecret
	// next 4 bytes -> nb elements in the vector (== nbPublic + nbSecret)
	bPublicWitness = bPublicWitness[12:]
	publicWitnessStr := hex.EncodeToString(bPublicWitness)

	checkerOpts = append(checkerOpts, "--dir", tmpDir)
	checkerOpts = append(checkerOpts, "--nb-public-inputs", strconv.Itoa(len(validPublicWitness.Vector().(fr_bn254.Vector))))
	checkerOpts = append(checkerOpts, "--proof", proofStr)
	checkerOpts = append(checkerOpts, "--public-inputs", publicWitnessStr)

	// verify proof
	// gnark-solidity-checker verify --dir tmdir --groth16 --nb-public-inputs 1 --proof 1234 --public-inputs dead
	cmd = exec.Command("gnark-solidity-checker", checkerOpts...)
	assert.t.Log("running ", cmd.String())
	out, err = cmd.CombinedOutput()
	assert.NoError(err, string(out))
}

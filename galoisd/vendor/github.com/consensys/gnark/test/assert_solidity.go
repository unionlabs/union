package test

import (
	"bytes"
	"encoding/hex"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"

	"github.com/consensys/gnark/backend"
	groth16_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	plonk_bn254 "github.com/consensys/gnark/backend/plonk/bn254"
	"github.com/consensys/gnark/backend/witness"
)

type verifyingKey interface {
	NbPublicWitness() int
	ExportSolidity(io.Writer) error
}

// solidityVerification checks that the exported solidity contract can verify the proof
// and that the proof is valid.
// It uses gnark-solidity-checker see test.WithSolidity option.
func (assert *Assert) solidityVerification(b backend.ID, vk verifyingKey,
	proof any,
	validPublicWitness witness.Witness) {
	if !SolcCheck || vk.NbPublicWitness() == 0 {
		return // nothing to check, will make solc fail.
	}
	assert.t.Helper()

	// make temp dir
	tmpDir, err := os.MkdirTemp("", "gnark-solidity-check*")
	assert.NoError(err)
	defer os.RemoveAll(tmpDir)

	// export solidity contract
	fSolidity, err := os.Create(filepath.Join(tmpDir, "gnark_verifier.sol"))
	assert.NoError(err)

	err = vk.ExportSolidity(fSolidity)
	assert.NoError(err)

	err = fSolidity.Close()
	assert.NoError(err)

	// generate assets
	// gnark-solidity-checker generate --dir tmpdir --solidity contract_g16.sol
	cmd := exec.Command("gnark-solidity-checker", "generate", "--dir", tmpDir, "--solidity", "gnark_verifier.sol")
	assert.t.Log("running ", cmd.String())
	out, err := cmd.CombinedOutput()
	assert.NoError(err, string(out))

	// proof to hex
	var proofStr string
	var optBackend string

	if b == backend.GROTH16 {
		optBackend = "--groth16"
		var buf bytes.Buffer
		_proof := proof.(*groth16_bn254.Proof)
		_, err = _proof.WriteRawTo(&buf)
		assert.NoError(err)
		proofBytes := buf.Bytes()
		// keep only fpSize * 8 bytes; for now solidity contract doesn't handle the commitment part.
		proofBytes = proofBytes[:32*8]
		proofStr = hex.EncodeToString(proofBytes)
	} else if b == backend.PLONK {
		optBackend = "--plonk"
		_proof := proof.(*plonk_bn254.Proof)
		// TODO @gbotrel make a single Marshal function for PlonK proof.
		proofStr = hex.EncodeToString(_proof.MarshalSolidity())
	} else {
		panic("not implemented")
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

	// verify proof
	// gnark-solidity-checker verify --dir tmdir --groth16 --nb-public-inputs 1 --proof 1234 --public-inputs dead
	cmd = exec.Command("gnark-solidity-checker", "verify",
		"--dir", tmpDir,
		optBackend,
		"--nb-public-inputs", strconv.Itoa(vk.NbPublicWitness()),
		"--proof", proofStr,
		"--public-inputs", publicWitnessStr)
	assert.t.Log("running ", cmd.String())
	out, err = cmd.CombinedOutput()
	assert.NoError(err, string(out))
}

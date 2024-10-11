package ics23

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

// TestVector is what is stored in the file
type TestVector struct {
	RootHash string `json:"root"`
	Proof    string `json:"proof"`
	Key      string `json:"key"`
	Value    string `json:"value"`
}

// RefData is parsed version of everything except the CommitmentProof itself
type RefData struct {
	RootHash []byte
	Key      []byte
	Value    []byte
}

type TestVectorsStruct struct {
	Dir      string
	Filename string
	Spec     *ProofSpec
}

func VectorsTestData() []TestVectorsStruct {
	iavl := filepath.Join("..", "testdata", "iavl")
	tendermint := filepath.Join("..", "testdata", "tendermint")
	smt := filepath.Join("..", "testdata", "smt")
	cases := []TestVectorsStruct{
		{Dir: iavl, Filename: "exist_left.json", Spec: IavlSpec},
		{Dir: iavl, Filename: "exist_right.json", Spec: IavlSpec},
		{Dir: iavl, Filename: "exist_middle.json", Spec: IavlSpec},
		{Dir: iavl, Filename: "nonexist_left.json", Spec: IavlSpec},
		{Dir: iavl, Filename: "nonexist_right.json", Spec: IavlSpec},
		{Dir: iavl, Filename: "nonexist_middle.json", Spec: IavlSpec},
		{Dir: tendermint, Filename: "exist_left.json", Spec: TendermintSpec},
		{Dir: tendermint, Filename: "exist_right.json", Spec: TendermintSpec},
		{Dir: tendermint, Filename: "exist_middle.json", Spec: TendermintSpec},
		{Dir: tendermint, Filename: "nonexist_left.json", Spec: TendermintSpec},
		{Dir: tendermint, Filename: "nonexist_right.json", Spec: TendermintSpec},
		{Dir: tendermint, Filename: "nonexist_middle.json", Spec: TendermintSpec},
		{Dir: smt, Filename: "exist_left.json", Spec: SmtSpec},
		{Dir: smt, Filename: "exist_right.json", Spec: SmtSpec},
		{Dir: smt, Filename: "exist_middle.json", Spec: SmtSpec},
		{Dir: smt, Filename: "nonexist_left.json", Spec: SmtSpec},
		{Dir: smt, Filename: "nonexist_right.json", Spec: SmtSpec},
		{Dir: smt, Filename: "nonexist_middle.json", Spec: SmtSpec},
	}
	return cases
}

// BatchVector is what is stored in the file
type BatchVector struct {
	RootHash string `json:"root"`
	Proof    string `json:"proof"`
	Items    []struct {
		Key   string `json:"key"`
		Value string `json:"value"`
	}
}

type BatchVectorData struct {
	Spec    *ProofSpec
	Proof   *CommitmentProof
	Ref     *RefData
	Invalid bool // default is valid
}

func BatchVectorsTestData(t *testing.T) map[string]BatchVectorData {
	t.Helper()
	iavl := filepath.Join("..", "testdata", "iavl")
	tendermint := filepath.Join("..", "testdata", "tendermint")
	smt := filepath.Join("..", "testdata", "smt")
	// Note that each item has a different commitment root,
	// so maybe not ideal (cannot check multiple entries)
	batchIAVL, refsIAVL := buildBatch(t, iavl, []string{
		"exist_left.json",
		"exist_right.json",
		"exist_middle.json",
		"nonexist_left.json",
		"nonexist_right.json",
		"nonexist_middle.json",
	})
	refsTML, refsTM := buildBatch(t, tendermint, []string{
		"exist_left.json",
		"exist_right.json",
		"exist_middle.json",
		"nonexist_left.json",
		"nonexist_right.json",
		"nonexist_middle.json",
	})
	batchSMT, refsSMT := buildBatch(t, smt, []string{
		"exist_left.json",
		"exist_right.json",
		"exist_middle.json",
		"nonexist_left.json",
		"nonexist_right.json",
		"nonexist_middle.json",
	})

	batchTMExist, refsTMExist := loadBatch(t, tendermint, "batch_exist.json")
	batchTMNonexist, refsTMNonexist := loadBatch(t, tendermint, "batch_nonexist.json")
	batchIAVLExist, refsIAVLExist := loadBatch(t, iavl, "batch_exist.json")
	batchIAVLNonexist, refsIAVLNonexist := loadBatch(t, iavl, "batch_nonexist.json")
	batchSMTexist, refsSMTexist := loadBatch(t, smt, "batch_exist.json")
	batchSMTnonexist, refsSMTnonexist := loadBatch(t, smt, "batch_nonexist.json")

	return map[string]BatchVectorData{
		"iavl 0": {Spec: IavlSpec, Proof: batchIAVL, Ref: refsIAVL[0]},
		"iavl 1": {Spec: IavlSpec, Proof: batchIAVL, Ref: refsIAVL[1]},
		"iavl 2": {Spec: IavlSpec, Proof: batchIAVL, Ref: refsIAVL[2]},
		"iavl 3": {Spec: IavlSpec, Proof: batchIAVL, Ref: refsIAVL[3]},
		"iavl 4": {Spec: IavlSpec, Proof: batchIAVL, Ref: refsIAVL[4]},
		"iavl 5": {Spec: IavlSpec, Proof: batchIAVL, Ref: refsIAVL[5]},
		// Note this spec only differs for non-existence proofs
		"iavl invalid 1":      {Spec: TendermintSpec, Proof: batchIAVL, Ref: refsIAVL[4], Invalid: true},
		"iavl invalid 2":      {Spec: IavlSpec, Proof: batchIAVL, Ref: refsTM[0], Invalid: true},
		"iavl batch exist":    {Spec: IavlSpec, Proof: batchIAVLExist, Ref: refsIAVLExist[17]},
		"iavl batch nonexist": {Spec: IavlSpec, Proof: batchIAVLNonexist, Ref: refsIAVLNonexist[7]},
		"tm 0":                {Spec: TendermintSpec, Proof: refsTML, Ref: refsTM[0]},
		"tm 1":                {Spec: TendermintSpec, Proof: refsTML, Ref: refsTM[1]},
		"tm 2":                {Spec: TendermintSpec, Proof: refsTML, Ref: refsTM[2]},
		"tm 3":                {Spec: TendermintSpec, Proof: refsTML, Ref: refsTM[3]},
		"tm 4":                {Spec: TendermintSpec, Proof: refsTML, Ref: refsTM[4]},
		"tm 5":                {Spec: TendermintSpec, Proof: refsTML, Ref: refsTM[5]},
		// Note this spec only differs for non-existence proofs
		"tm invalid 1":      {Spec: IavlSpec, Proof: refsTML, Ref: refsTM[4], Invalid: true},
		"tm invalid 2":      {Spec: TendermintSpec, Proof: refsTML, Ref: refsIAVL[0], Invalid: true},
		"tm batch exist":    {Spec: TendermintSpec, Proof: batchTMExist, Ref: refsTMExist[10]},
		"tm batch nonexist": {Spec: TendermintSpec, Proof: batchTMNonexist, Ref: refsTMNonexist[3]},
		"smt 0":             {Spec: SmtSpec, Proof: batchSMT, Ref: refsSMT[0]},
		"smt 1":             {Spec: SmtSpec, Proof: batchSMT, Ref: refsSMT[1]},
		"smt 2":             {Spec: SmtSpec, Proof: batchSMT, Ref: refsSMT[2]},
		"smt 3":             {Spec: SmtSpec, Proof: batchSMT, Ref: refsSMT[3]},
		"smt 4":             {Spec: SmtSpec, Proof: batchSMT, Ref: refsSMT[4]},
		"smt 5":             {Spec: SmtSpec, Proof: batchSMT, Ref: refsSMT[5]},
		// Note this spec only differs for non-existence proofs
		"smt invalid 1":      {Spec: IavlSpec, Proof: batchSMT, Ref: refsSMT[4], Invalid: true},
		"smt invalid 2":      {Spec: SmtSpec, Proof: batchSMT, Ref: refsIAVL[0], Invalid: true},
		"smt batch exist":    {Spec: SmtSpec, Proof: batchSMTexist, Ref: refsSMTexist[10]},
		"smt batch nonexist": {Spec: SmtSpec, Proof: batchSMTnonexist, Ref: refsSMTnonexist[3]},
	}
}

func DecompressBatchVectorsTestData(t *testing.T) map[string]*CommitmentProof {
	t.Helper()
	iavl := filepath.Join("..", "testdata", "iavl")
	tendermint := filepath.Join("..", "testdata", "tendermint")
	smt := filepath.Join("..", "testdata", "smt")
	// note that these batches are already compressed
	batchIAVL, _ := loadBatch(t, iavl, "batch_exist.json")
	batchTM, _ := loadBatch(t, tendermint, "batch_nonexist.json")
	batchSMT, _ := loadBatch(t, smt, "batch_nonexist.json")
	return map[string]*CommitmentProof{
		"iavl":       batchIAVL,
		"tendermint": batchTM,
		"smt":        batchSMT,
	}
}

func LoadFile(tb testing.TB, dir string, filename string) (*CommitmentProof, *RefData) {
	tb.Helper()
	// load the file into a json struct
	name := filepath.Join(dir, filename)
	bz, err := os.ReadFile(name)
	if err != nil {
		tb.Fatalf("Read file: %+v", err)
	}
	var data TestVector
	err = json.Unmarshal(bz, &data)
	if err != nil {
		tb.Fatalf("Unmarshal json: %+v", err)
	}
	// parse the protobuf object
	var proof CommitmentProof
	err = proof.Unmarshal(mustHex(tb, data.Proof))
	if err != nil {
		tb.Fatalf("Unmarshal protobuf: %+v", err)
	}
	var ref RefData
	ref.RootHash = CommitmentRoot(mustHex(tb, data.RootHash))
	ref.Key = mustHex(tb, data.Key)
	if data.Value != "" {
		ref.Value = mustHex(tb, data.Value)
	}
	return &proof, &ref
}

func mustHex(tb testing.TB, data string) []byte {
	tb.Helper()
	if data == "" {
		return nil
	}
	res, err := hex.DecodeString(data)
	if err != nil {
		tb.Fatalf("decoding hex: %v", err)
	}
	return res
}

func buildBatch(t *testing.T, dir string, filenames []string) (*CommitmentProof, []*RefData) {
	t.Helper()
	refs := make([]*RefData, len(filenames))
	proofs := make([]*CommitmentProof, len(filenames))
	for i, fn := range filenames {
		proofs[i], refs[i] = LoadFile(t, dir, fn)
	}
	batch, err := CombineProofs(proofs)
	if err != nil {
		t.Fatalf("Generating batch: %v", err)
	}
	return batch, refs
}

func loadBatch(t *testing.T, dir string, filename string) (*CommitmentProof, []*RefData) {
	t.Helper()
	// load the file into a json struct
	name := filepath.Join(dir, filename)
	bz, err := os.ReadFile(name)
	if err != nil {
		t.Fatalf("Read file: %+v", err)
	}
	var data BatchVector
	err = json.Unmarshal(bz, &data)
	if err != nil {
		t.Fatalf("Unmarshal json: %+v", err)
	}
	// parse the protobuf object
	var proof CommitmentProof
	err = proof.Unmarshal(mustHex(t, data.Proof))
	if err != nil {
		t.Fatalf("Unmarshal protobuf: %+v", err)
	}
	root := mustHex(t, data.RootHash)
	refs := make([]*RefData, len(data.Items))
	for i, item := range data.Items {
		refs[i] = &RefData{
			RootHash: root,
			Key:      mustHex(t, item.Key),
			Value:    mustHex(t, item.Value),
		}
	}
	return &proof, refs
}

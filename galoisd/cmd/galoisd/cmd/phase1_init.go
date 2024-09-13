package cmd

import (
	"crypto/sha256"

	"github.com/spf13/cobra"

	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math"
	"math/big"
	"os"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	mpc "github.com/consensys/gnark/backend/groth16/bn254/mpcsetup"
)

func Phase1InitCmd() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Initialize the phase 1 of the groth16 multi-party computation.",
		Use:   "mpc-phase1-init [ptau] [phase1FinalOutput]",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			ptauPath := args[0]
			ptau, err := ReadPtau(ptauPath)
			if err != nil {
				return err
			}
			srs1, err := convertPtauToPhase1(ptau)
			if err != nil {
				return err
			}
			phase1FinalPath := args[1]
			return saveTo(phase1FinalPath, &srs1)
		},
	}
	return cmd
}

///////////////////////////////////////////////////////////////////
///                             PTAU                            ///
///////////////////////////////////////////////////////////////////
// Format
// Taken from the iden3/snarkjs repo powersoftau_new.js file
// https://github.com/iden3/snarkjs/blob/master/src/powersoftau_new.js
/*
Header(1)
    n8
    prime
    power
tauG1(2)
    {(2 ** power)*2-1} [
        G1, tau*G1, tau^2 * G1, ....
    ]
tauG2(3)
    {2 ** power}[
        G2, tau*G2, tau^2 * G2, ...
    ]
alphaTauG1(4)
    {2 ** power}[
        alpha*G1, alpha*tau*G1, alpha*tau^2*G1,....
    ]
betaTauG1(5)
    {2 ** power} []
        beta*G1, beta*tau*G1, beta*tau^2*G1, ....
    ]
betaG2(6)
    {1}[
        beta*G2
    ]
contributions(7) - Ignore contributions, users can verify using snarkjs
    NContributions
    {NContributions}[
        tau*G1
        tau*G2
        alpha*G1
        beta*G1
        beta*G2
        pubKey
            tau_g1s
            tau_g1sx
            tau_g2spx
            alpha_g1s
            alpha_g1sx
            alpha_g1spx
            beta_g1s
            beta_g1sx
            beta_g1spx
        partialHash (216 bytes) See https://github.com/mafintosh/blake2b-wasm/blob/23bee06945806309977af802bc374727542617c7/blake2b.wat#L9
        hashNewChallenge
    ]
*/

// in bytes
const BN254_FIELD_ELEMENT_SIZE = 32

type G1 [2]big.Int
type G2 [4]big.Int

type PtauHeader struct {
	N8    uint32
	Prime big.Int
	Power uint32
}

type Ptau struct {
	Header     PtauHeader
	PTauPubKey PtauPubKey
}

type PtauPubKey struct {
	TauG1      []G1
	TauG2      []G2
	AlphaTauG1 []G1
	BetaTauG1  []G1
	BetaG2     G2
}

type PtauFile struct {
	Header   PtauHeader
	Sections [][]SectionSegment
	Reader   *os.File
}

func InitPtau(path string) (*PtauFile, error) {
	reader, err := os.Open(path)

	if err != nil {
		return nil, err
	}

	var ptauStr = make([]byte, 4)
	_, err = reader.Read(ptauStr)

	// version
	_, err = readULE32(reader)

	// number of sections
	_, err = readULE32(reader)

	numSections := uint32(7)

	// in practice, all sections have only one segment, but who knows...
	// 1-based indexing, so we need to allocate one more than the number of sections
	sections := make([][]SectionSegment, numSections+1)
	for i := uint32(0); i < numSections; i++ {
		ht, _ := readULE32(reader)
		hl, _ := readULE64(reader)
		if sections[ht] == nil {
			sections[ht] = make([]SectionSegment, 0)
		}
		pos, _ := reader.Seek(0, io.SeekCurrent)
		sections[ht] = append(sections[ht], SectionSegment{pos: uint64(pos), size: hl})
		reader.Seek(int64(hl), io.SeekCurrent)
	}

	// section size
	_, err = readBigInt(reader, 8)

	// Header (1)
	seekToUniqueSection(reader, sections, 1)

	// Read header
	header, err := readPtauHeader(reader)

	if err != nil {
		return nil, err
	}

	return &PtauFile{Header: header, Sections: sections, Reader: reader}, nil
}

func (ptauFile *PtauFile) Close() error {
	return ptauFile.Reader.Close()
}

func (ptauFile *PtauFile) DomainSize() int {
	return 1 << ptauFile.Header.Power
}

func (ptauFile *PtauFile) readG1s(out chan bn254.G1Affine, count int) error {
	for i := 0; i < count; i++ {
		g1, err := readG1(ptauFile.Reader)
		if err != nil {
			return err
		}
		g1Affine := bn254.G1Affine{}
		x := bytesToElement(g1[0].Bytes())
		g1Affine.X = x
		y := bytesToElement(g1[1].Bytes())
		g1Affine.Y = y
		if !g1Affine.IsOnCurve() {
			panic("g1Affine is not on curve")
		}
		out <- g1Affine
	}
	return nil
}

func (ptauFile *PtauFile) readG2() (bn254.G2Affine, error) {
	g2, err := readG2(ptauFile.Reader)
	if err != nil {
		return bn254.G2Affine{}, err
	}
	g2Affine := bn254.G2Affine{}
	x0 := bytesToElement(g2[0].Bytes())
	x1 := bytesToElement(g2[1].Bytes())
	g2Affine.X.A0 = x0
	g2Affine.X.A1 = x1
	y0 := bytesToElement(g2[2].Bytes())
	y1 := bytesToElement(g2[3].Bytes())
	g2Affine.Y.A0 = y0
	g2Affine.Y.A1 = y1
	if !g2Affine.IsOnCurve() {

		panic("g2Affine is not on curve")
	}
	return g2Affine, nil
}

func (ptauFile *PtauFile) readG2s(out chan bn254.G2Affine, count int) error {
	for i := 0; i < count; i++ {
		g2Affine, err := ptauFile.readG2()
		if err != nil {
			return err
		}
		out <- g2Affine
	}
	return nil
}

func (ptauFile *PtauFile) ReadTauG1(out chan bn254.G1Affine) error {
	defer close(out)
	seekToUniqueSection(ptauFile.Reader, ptauFile.Sections, 2)
	numPoints := ptauFile.DomainSize()*2 - 1
	ptauFile.readG1s(out, numPoints)
	return nil
}

func (ptauFile *PtauFile) ReadTauG2(out chan bn254.G2Affine) error {
	defer close(out)
	seekToUniqueSection(ptauFile.Reader, ptauFile.Sections, 3)
	numPoints := ptauFile.DomainSize()
	ptauFile.readG2s(out, numPoints)
	return nil
}

func (ptauFile *PtauFile) ReadAlphaTauG1(out chan bn254.G1Affine) error {
	defer close(out)
	seekToUniqueSection(ptauFile.Reader, ptauFile.Sections, 4)
	numPoints := ptauFile.DomainSize()
	ptauFile.readG1s(out, numPoints)
	return nil
}

func (ptauFile *PtauFile) ReadBetaTauG1(out chan bn254.G1Affine) error {
	defer close(out)
	seekToUniqueSection(ptauFile.Reader, ptauFile.Sections, 5)
	numPoints := ptauFile.DomainSize()
	ptauFile.readG1s(out, numPoints)
	return nil
}

func (ptauFile *PtauFile) ReadBetaG2() (bn254.G2Affine, error) {
	seekToUniqueSection(ptauFile.Reader, ptauFile.Sections, 6)
	return ptauFile.readG2()
}

func ReadPtau(zkeyPath string) (Ptau, error) {
	reader, err := os.Open(zkeyPath)
	if err != nil {
		return Ptau{}, err
	}

	defer reader.Close()

	var ptauStr = make([]byte, 4)
	_, err = reader.Read(ptauStr)
	if err != nil {
		return Ptau{}, err
	}

	// version
	_, err = readULE32(reader)
	if err != nil {
		return Ptau{}, err
	}

	// number of sections
	_, err = readULE32(reader)
	if err != nil {
		return Ptau{}, err
	}

	numSections := uint32(7)

	// in practice, all sections have only one segment, but who knows...
	// 1-based indexing, so we need to allocate one more than the number of sections
	sections := make([][]SectionSegment, numSections+1)
	for i := uint32(0); i < numSections; i++ {
		ht, err := readULE32(reader)
		if err != nil {
			return Ptau{}, err
		}
		hl, err := readULE64(reader)
		if err != nil {
			return Ptau{}, err
		}
		if sections[ht] == nil {
			sections[ht] = make([]SectionSegment, 0)
		}
		pos, err := reader.Seek(0, io.SeekCurrent)
		if err != nil {
			return Ptau{}, err
		}
		sections[ht] = append(sections[ht], SectionSegment{pos: uint64(pos), size: hl})
		reader.Seek(int64(hl), io.SeekCurrent)
	}

	// section size
	_, err = readBigInt(reader, 8)
	if err != nil {
		return Ptau{}, err
	}

	// Header (1)
	seekToUniqueSection(reader, sections, 1)

	// Read header
	header, err := readPtauHeader(reader)
	if err != nil {
		return Ptau{}, err
	}

	// TauG1 (2)
	seekToUniqueSection(reader, sections, 2)

	var PtauPubKey PtauPubKey

	twoToPower := uint32(1 << header.Power)

	PtauPubKey.TauG1, err = readG1Array(reader, twoToPower*2-1)
	if err != nil {
		return Ptau{}, err
	}

	// TauG2 (3)
	seekToUniqueSection(reader, sections, 3)

	PtauPubKey.TauG2, err = readG2Array(reader, twoToPower)
	if err != nil {
		return Ptau{}, err
	}

	// AlphaTauG1 (4)
	seekToUniqueSection(reader, sections, 4)

	PtauPubKey.AlphaTauG1, err = readG1Array(reader, twoToPower)
	if err != nil {
		return Ptau{}, err
	}

	// BetaTauG1 (5)
	seekToUniqueSection(reader, sections, 5)

	PtauPubKey.BetaTauG1, err = readG1Array(reader, twoToPower)
	if err != nil {
		return Ptau{}, err
	}

	// BetaG2 (6)
	seekToUniqueSection(reader, sections, 6)

	PtauPubKey.BetaG2, err = readG2(reader)
	if err != nil {
		return Ptau{}, err
	}

	return Ptau{Header: header, PTauPubKey: PtauPubKey}, nil
}

func readPtauHeader(reader io.ReadSeeker) (PtauHeader, error) {
	var header PtauHeader
	n8, err := readULE32(reader)
	if err != nil {
		return PtauHeader{}, err
	}

	header.N8 = n8

	prime, err := readBigInt(reader, n8)
	if err != nil {
		return PtauHeader{}, err
	}

	header.Prime = prime

	power, err := readULE32(reader)
	if err != nil {
		return PtauHeader{}, err
	}

	header.Power = power
	return header, nil
}

func readG1Array(reader io.ReadSeeker, numPoints uint32) ([]G1, error) {
	g1s := make([]G1, numPoints)
	for i := uint32(0); i < numPoints; i++ {
		g1, err := readG1(reader)
		if err != nil {
			return []G1{}, err
		}

		g1s[i] = g1
	}
	return g1s, nil
}

func readG2Array(reader io.ReadSeeker, numPoints uint32) ([]G2, error) {
	g2s := make([]G2, numPoints)

	for i := uint32(0); i < numPoints; i++ {
		g2, err := readG2(reader)

		if err != nil {
			return []G2{}, err
		}

		g2s[i] = g2
	}

	return g2s, nil
}

func readTauG2(reader io.ReadSeeker) ([]G2, error) {
	tauG2_s, err := readG2(reader)
	if err != nil {
		return []G2{}, err
	}

	tauG2_sx, err := readG2(reader)
	if err != nil {
		return []G2{}, err
	}

	return []G2{tauG2_s, tauG2_sx}, nil
}

func readG1(reader io.ReadSeeker) (G1, error) {
	var g1 G1

	x, err := readBigInt(reader, BN254_FIELD_ELEMENT_SIZE)
	if err != nil {
		return G1{}, err
	}

	g1[0] = x

	y, err := readBigInt(reader, BN254_FIELD_ELEMENT_SIZE)
	if err != nil {
		return G1{}, err
	}

	g1[1] = y

	return g1, nil
}

func readG2(reader io.ReadSeeker) (G2, error) {
	var g2 G2

	x0, err := readBigInt(reader, BN254_FIELD_ELEMENT_SIZE)
	if err != nil {
		return G2{}, err
	}

	g2[0] = x0

	x1, err := readBigInt(reader, BN254_FIELD_ELEMENT_SIZE)
	if err != nil {
		return G2{}, err
	}

	g2[1] = x1

	y0, err := readBigInt(reader, BN254_FIELD_ELEMENT_SIZE)
	if err != nil {
		return G2{}, err
	}

	g2[2] = y0

	y1, err := readBigInt(reader, BN254_FIELD_ELEMENT_SIZE)
	if err != nil {
		return G2{}, err
	}

	g2[3] = y1

	return g2, nil
}

func readULE32(reader io.Reader) (uint32, error) {
	var buffer = make([]byte, 4)

	_, err := reader.Read(buffer)
	if err != nil {
		return 0, err
	}

	return binary.LittleEndian.Uint32(buffer), nil
}

func readULE64(reader io.Reader) (uint64, error) {
	var buffer = make([]byte, 8)

	_, err := reader.Read(buffer)
	if err != nil {
		return 0, err
	}

	return binary.LittleEndian.Uint64(buffer), nil
}

func readBigInt(reader io.Reader, n8 uint32) (big.Int, error) {
	var buffer = make([]byte, n8)

	_, err := reader.Read(buffer)
	reverseSlice(buffer)

	if err != nil {
		return *big.NewInt(0), err
	}

	bigInt := big.NewInt(0).SetBytes(buffer)

	return *bigInt, nil
}

func reverseSlice(slice []byte) []byte {
	for i := 0; i < len(slice)/2; i++ {
		j := len(slice) - i - 1
		slice[i], slice[j] = slice[j], slice[i]
	}
	return slice
}

func bytesToElement(b []byte) fp.Element {
	var z fp.Element
	reverseSlice(b)
	if len(b) < 32 {
		b = append(b, make([]byte, 32-len(b))...)
	}

	z[0] = binary.LittleEndian.Uint64(b[0:8])
	z[1] = binary.LittleEndian.Uint64(b[8:16])
	z[2] = binary.LittleEndian.Uint64(b[16:24])
	z[3] = binary.LittleEndian.Uint64(b[24:32])

	return z
}

///////////////////////////////////////////////////////////////////
///                             ZKEY                            ///
///////////////////////////////////////////////////////////////////

// Taken from the iden3/snarkjs repo, zkey_utils.js
// (https://github.com/iden3/snarkjs/blob/fb144555d8ce4779ad79e707f269771c672a8fb7/src/zkey_utils.js#L20-L45)
// Format
// ======
// 4 bytes, zket
// 4 bytes, version
// 4 bytes, number of sections
// 4 bytes, section number
// 8 bytes, section size
// Header(1)
// 4 bytes, Prover Type 1 Groth
// HeaderGroth(2)
// 4 bytes, n8q
// n8q bytes, q
// 4 bytes, n8r
// n8r bytes, r
// 4 bytes, NVars
// 4 bytes, NPub
// 4 bytes, DomainSize  (multiple of 2)
//      alpha1
//      beta1
//      delta1
//      beta2
//      gamma2
//      delta2

const GROTH_16_PROTOCOL_ID = uint32(1)

type NotGroth16 struct {
	Err error
}

func (r *NotGroth16) Error() string {
	return fmt.Sprintf("Groth16 is the only supported protocol at this time (PLONK and FFLONK are not): %v", r.Err)
}

// Incomplete (only extracts necessary fields for conversion to .ph1 format)
type Zkey struct {
	ZkeyHeader     ZkeyHeader
	protocolHeader HeaderGroth
}

type ZkeyHeader struct {
	ProtocolID     uint32
	protocolHeader HeaderGroth
}

type HeaderGroth struct {
	n8q        uint32
	q          big.Int
	n8r        uint32
	r          big.Int
	nVars      uint32
	nPublic    uint32
	domainSize uint32
	power      uint32
}

type SectionSegment struct {
	pos  uint64
	size uint64
}

func ReadZkey(zkeyPath string) (Zkey, error) {
	reader, err := os.Open(zkeyPath)

	if err != nil {
		return Zkey{}, err
	}

	defer reader.Close()

	// zkey
	var zkeyStr = make([]byte, 4)
	_, err = reader.Read(zkeyStr)
	if err != nil {
		return Zkey{}, err
	}

	// version
	_, err = readULE32(reader)
	if err != nil {
		return Zkey{}, err
	}

	// number of sections
	numSections, err := readULE32(reader)

	// in practice, all sections have only one segment, but who knows...
	// 1-based indexing, so we need to allocate one more than the number of sections
	sections := make([][]SectionSegment, numSections+1)
	for i := uint32(0); i < numSections; i++ {
		ht, _ := readULE32(reader)
		hl, _ := readULE64(reader)
		if sections[ht] == nil {
			sections[ht] = make([]SectionSegment, 0)
		}
		pos, _ := reader.Seek(0, io.SeekCurrent)
		sections[ht] = append(sections[ht], SectionSegment{pos: uint64(pos), size: hl})
		reader.Seek(int64(hl), io.SeekCurrent)
	}

	// section size
	_, err = readBigInt(reader, 8)
	if err != nil {
		return Zkey{}, err
	}

	seekToUniqueSection(reader, sections, 1)
	header, err := readHeader(reader, sections)
	if err != nil {
		return Zkey{}, err
	}

	zkey := Zkey{ZkeyHeader: header, protocolHeader: header.protocolHeader}

	return zkey, nil
}

func seekToUniqueSection(reader io.ReadSeeker, sections [][]SectionSegment, sectionId uint32) {
	section := sections[sectionId]

	if len(section) > 1 {
		panic("Section has more than one segment")
	}

	reader.Seek(int64(section[0].pos), io.SeekStart)
}

func readHeader(reader io.ReadSeeker, sections [][]SectionSegment) (ZkeyHeader, error) {
	var header = ZkeyHeader{}

	protocolID, err := readULE32(reader)

	if err != nil {
		return header, err
	}

	// if groth16
	if protocolID == GROTH_16_PROTOCOL_ID {
		seekToUniqueSection(reader, sections, 2)
		headerGroth, err := readHeaderGroth16(reader)

		if err != nil {
			return header, err
		}

		header = ZkeyHeader{ProtocolID: protocolID, protocolHeader: headerGroth}

	} else {
		return header, &NotGroth16{Err: errors.New("ProtocolID is not Groth16")}
	}

	return header, nil
}

func readHeaderGroth16(reader io.ReadSeeker) (HeaderGroth, error) {
	var header = HeaderGroth{}

	n8q, err := readULE32(reader)
	if err != nil {
		return header, err
	}

	q, err := readBigInt(reader, n8q)
	if err != nil {
		return header, err
	}

	n8r, err := readULE32(reader)
	if err != nil {
		return header, err
	}

	r, err := readBigInt(reader, n8r)
	if err != nil {
		return header, err
	}

	nVars, err := readULE32(reader)
	if err != nil {
		return header, err
	}

	nPublic, err := readULE32(reader)
	if err != nil {
		return header, err
	}

	domainSize, err := readULE32(reader)
	if err != nil {
		return header, err
	}

	power := math.Log2(float64(domainSize))

	power_int := uint32(math.Ceil(power))

	header = HeaderGroth{n8q: n8q, q: q, n8r: n8r, r: r, nVars: nVars, nPublic: nPublic, domainSize: domainSize, power: power_int}

	return header, nil
}

func convertPtauToPhase1(ptau Ptau) (phase1 mpc.Phase1, err error) {
	tauG1 := make([]bn254.G1Affine, len(ptau.PTauPubKey.TauG1))
	for i, g1 := range ptau.PTauPubKey.TauG1 {
		g1Affine := bn254.G1Affine{}
		x := bytesToElement(g1[0].Bytes())
		g1Affine.X = x
		y := bytesToElement(g1[1].Bytes())
		g1Affine.Y = y
		if !g1Affine.IsOnCurve() {
			fmt.Printf("tauG1: \n index: %v g1Affine.X: %v \n g1Affine.Y: %v \n", i, g1Affine.X.String(), g1Affine.Y.String())
			panic("g1Affine is not on curve")
		}
		tauG1[i] = g1Affine
	}

	alphaTauG1 := make([]bn254.G1Affine, len(ptau.PTauPubKey.AlphaTauG1))
	for i, g1 := range ptau.PTauPubKey.AlphaTauG1 {
		g1Affine := bn254.G1Affine{}
		x := bytesToElement(g1[0].Bytes())
		g1Affine.X = x
		y := bytesToElement(g1[1].Bytes())
		g1Affine.Y = y
		if !g1Affine.IsOnCurve() {
			fmt.Printf("alphaTauG1: \n index: %v g1Affine.X: %v \n g1Affine.Y: %v \n", i, g1Affine.X.String(), g1Affine.Y.String())
			panic("g1Affine is not on curve")
		}
		alphaTauG1[i] = g1Affine
	}
	// fmt.Printf("alphaTauG1: %v \n", alphaTauG1)

	betaTauG1 := make([]bn254.G1Affine, len(ptau.PTauPubKey.BetaTauG1))

	for i, g1 := range ptau.PTauPubKey.BetaTauG1 {
		g1Affine := bn254.G1Affine{}
		x := bytesToElement(g1[0].Bytes())
		g1Affine.X = x
		y := bytesToElement(g1[1].Bytes())
		g1Affine.Y = y
		if !g1Affine.IsOnCurve() {
			fmt.Printf("betaTauG1: \n index: %v, g1Affine.X: %v \n g1Affine.Y: %v \n", i, g1Affine.X.String(), g1Affine.Y.String())
			panic("g1Affine is not on curve")
		}
		betaTauG1[i] = g1Affine
	}
	tauG2 := make([]bn254.G2Affine, len(ptau.PTauPubKey.TauG2))
	for i, g2 := range ptau.PTauPubKey.TauG2 {
		g2Affine := bn254.G2Affine{}
		x0 := bytesToElement(g2[0].Bytes())
		x1 := bytesToElement(g2[1].Bytes())
		g2Affine.X.A0 = x0
		g2Affine.X.A1 = x1
		y0 := bytesToElement(g2[2].Bytes())
		y1 := bytesToElement(g2[3].Bytes())
		g2Affine.Y.A0 = y0
		g2Affine.Y.A1 = y1
		if !g2Affine.IsOnCurve() {
			fmt.Printf("tauG2: \n index: %v, g2Affine.X.A0: %v \n g2Affine.X.A1: %v \n g2Affine.Y.A0: %v \n g2Affine.Y.A1 %v \n", i, g2Affine.X.A0.String(), g2Affine.X.A1.String(), g2Affine.Y.A0.String(), g2Affine.Y.A1.String())
			panic("g2Affine is not on curve")
		}
		tauG2[i] = g2Affine
	}

	betaG2 := bn254.G2Affine{}
	{
		g2 := ptau.PTauPubKey.BetaG2

		x0 := bytesToElement(g2[0].Bytes())
		x1 := bytesToElement(g2[1].Bytes())
		betaG2.X.A0 = x0
		betaG2.X.A1 = x1
		y0 := bytesToElement(g2[2].Bytes())
		y1 := bytesToElement(g2[3].Bytes())
		betaG2.Y.A0 = y0
		betaG2.Y.A1 = y1

		if !betaG2.IsOnCurve() {
			fmt.Printf("g2Affine.X.A0: %v \n g2Affine.X.A1: %v \n g2Affine.Y.A0: %v \n g2Affine.Y.A1 %v \n", betaG2.X.A0.String(), betaG2.X.String(), betaG2.Y.A0.String(), betaG2.Y.A1.String())
			panic("g2Affine is not on curve")
		}
	}

	phase1 = mpc.InitPhase1(int(ptau.Header.Power))

	phase1.Parameters.G1.Tau = tauG1
	phase1.Parameters.G1.AlphaTau = alphaTauG1
	phase1.Parameters.G1.BetaTau = betaTauG1

	phase1.Parameters.G2.Tau = tauG2
	phase1.Parameters.G2.Beta = betaG2

	sha := sha256.New()
	phase1.WriteTo(sha)
	phase1.Hash = sha.Sum(nil)

	return phase1, nil
}

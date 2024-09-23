package constraint

import (
	"fmt"
	"math/big"
	"strconv"
	"sync"

	"github.com/blang/semver/v4"
	"github.com/consensys/gnark"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/internal/tinyfield"
	"github.com/consensys/gnark/internal/utils"
	"github.com/consensys/gnark/logger"
	"github.com/consensys/gnark/profile"
)

type SystemType uint16

const (
	SystemUnknown SystemType = iota
	SystemR1CS
	SystemSparseR1CS
)

// PackedInstruction is the lowest element of a constraint system. It stores just enough data to
// reconstruct a constraint of any shape or a hint at solving time.
type PackedInstruction struct {
	// BlueprintID maps this instruction to a blueprint
	BlueprintID BlueprintID

	// ConstraintOffset stores the starting constraint ID of this instruction.
	// Might not be strictly necessary; but speeds up solver for instructions that represents
	// multiple constraints.
	ConstraintOffset uint32

	// WireOffset stores the starting internal wire ID of this instruction. Blueprints may use this
	// and refer to output wires by their offset.
	// For example, if a blueprint declared 5 outputs, the first output wire will be WireOffset,
	// the last one WireOffset+4.
	WireOffset uint32

	// The constraint system stores a single []uint32 calldata slice. StartCallData
	// points to the starting index in the mentioned slice. This avoid storing a slice per
	// instruction (3 * uint64 in memory).
	StartCallData uint64
}

// Unpack returns the instruction corresponding to the packed instruction.
func (pi PackedInstruction) Unpack(cs *System) Instruction {

	blueprint := cs.Blueprints[pi.BlueprintID]
	cSize := blueprint.CalldataSize()
	if cSize < 0 {
		// by convention, we store nbInputs < 0 for non-static input length.
		cSize = int(cs.CallData[pi.StartCallData])
	}

	return Instruction{
		ConstraintOffset: pi.ConstraintOffset,
		WireOffset:       pi.WireOffset,
		Calldata:         cs.CallData[pi.StartCallData : pi.StartCallData+uint64(cSize)],
	}
}

// Instruction is the lowest element of a constraint system. It stores all the data needed to
// reconstruct a constraint of any shape or a hint at solving time.
type Instruction struct {
	ConstraintOffset uint32
	WireOffset       uint32
	Calldata         []uint32
}

// System contains core elements for a constraint System
type System struct {
	// serialization header
	GnarkVersion string
	ScalarField  string

	Type SystemType

	Instructions []PackedInstruction `cbor:"-"`
	Blueprints   []Blueprint
	CallData     []uint32 `cbor:"-"`

	// can be != than len(instructions)
	NbConstraints int

	// number of internal wires
	NbInternalVariables int

	// input wires names
	Public, Secret []string

	// logs (added with system.Println, resolved when solver sets a value to a wire)
	Logs []LogEntry

	// debug info contains stack trace (including line number) of a call to a system.API that
	// results in an unsolved constraint
	DebugInfo   []LogEntry
	SymbolTable debug.SymbolTable
	// maps constraint id to debugInfo id
	// several constraints may point to the same debug info
	MDebug map[int]int

	// maps hintID to hint string identifier
	MHintsDependencies map[solver.HintID]string

	// each level contains independent constraints and can be parallelized
	// it is guaranteed that all dependencies for constraints in a level l are solved
	// in previous levels
	// TODO @gbotrel these are currently updated after we add a constraint.
	// but in case the object is built from a serialized representation
	// we need to init the level builder lbWireLevel from the existing constraints.
	Levels [][]uint32 `cbor:"-"`

	// scalar field
	q      *big.Int `cbor:"-"`
	bitLen int      `cbor:"-"`

	// level builder
	lbWireLevel []Level `cbor:"-"` // at which level we solve a wire. init at -1.

	CommitmentInfo Commitments
	GkrInfo        GkrInfo

	genericHint BlueprintID
}

// NewSystem initialize the common structure among constraint system
func NewSystem(scalarField *big.Int, capacity int, t SystemType) System {
	system := System{
		Type:               t,
		SymbolTable:        debug.NewSymbolTable(),
		MDebug:             map[int]int{},
		GnarkVersion:       gnark.Version.String(),
		ScalarField:        scalarField.Text(16),
		MHintsDependencies: make(map[solver.HintID]string),
		q:                  new(big.Int).Set(scalarField),
		bitLen:             scalarField.BitLen(),
		Instructions:       make([]PackedInstruction, 0, capacity),
		CallData:           make([]uint32, 0, capacity*8),
		lbWireLevel:        make([]Level, 0, capacity),
		Levels:             make([][]uint32, 0, capacity/2),
		CommitmentInfo:     NewCommitments(t),
	}

	system.genericHint = system.AddBlueprint(&BlueprintGenericHint{})
	return system
}

// GetNbInstructions returns the number of instructions in the system
func (system *System) GetNbInstructions() int {
	return len(system.Instructions)
}

// GetInstruction returns the instruction at index id
func (system *System) GetInstruction(id int) Instruction {
	return system.Instructions[id].Unpack(system)
}

// AddBlueprint adds a blueprint to the system and returns its ID
func (system *System) AddBlueprint(b Blueprint) BlueprintID {
	system.Blueprints = append(system.Blueprints, b)
	return BlueprintID(len(system.Blueprints) - 1)
}

func (system *System) GetNbSecretVariables() int {
	return len(system.Secret)
}
func (system *System) GetNbPublicVariables() int {
	return len(system.Public)
}
func (system *System) GetNbInternalVariables() int {
	return system.NbInternalVariables
}

// CheckSerializationHeader parses the scalar field and gnark version headers
//
// This is meant to be use at the deserialization step, and will error for illegal values
func (system *System) CheckSerializationHeader() error {
	// check gnark version
	binaryVersion := gnark.Version
	objectVersion, err := semver.Parse(system.GnarkVersion)
	if err != nil {
		return fmt.Errorf("when parsing gnark version: %w", err)
	}

	if binaryVersion.Compare(objectVersion) != 0 {
		log := logger.Logger()
		log.Warn().Str("binary", binaryVersion.String()).Str("object", objectVersion.String()).Msg("gnark version (binary) mismatch with constraint system. there are no guarantees on compatibility")
	}

	// TODO @gbotrel maintain version changes and compare versions properly
	// (ie if major didn't change,we shouldn't have a compatibility issue)

	scalarField := new(big.Int)
	_, ok := scalarField.SetString(system.ScalarField, 16)
	if !ok {
		return fmt.Errorf("when parsing serialized modulus: %s", system.ScalarField)
	}
	curveID := utils.FieldToCurve(scalarField)
	if curveID == ecc.UNKNOWN && scalarField.Cmp(tinyfield.Modulus()) != 0 {
		return fmt.Errorf("unsupported scalar field %s", scalarField.Text(16))
	}
	system.q = new(big.Int).Set(scalarField)
	system.bitLen = system.q.BitLen()
	return nil
}

// GetNbVariables return number of internal, secret and public variables
func (system *System) GetNbVariables() (internal, secret, public int) {
	return system.NbInternalVariables, system.GetNbSecretVariables(), system.GetNbPublicVariables()
}

func (system *System) Field() *big.Int {
	return new(big.Int).Set(system.q)
}

// bitLen returns the number of bits needed to represent a fr.Element
func (system *System) FieldBitLen() int {
	return system.bitLen
}

func (system *System) AddInternalVariable() (idx int) {
	idx = system.NbInternalVariables + system.GetNbPublicVariables() + system.GetNbSecretVariables()
	system.NbInternalVariables++
	// also grow the level slice
	system.lbWireLevel = append(system.lbWireLevel, LevelUnset)
	if debug.Debug && len(system.lbWireLevel) != system.NbInternalVariables {
		panic("internal error")
	}
	return idx
}

func (system *System) AddPublicVariable(name string) (idx int) {
	idx = system.GetNbPublicVariables()
	system.Public = append(system.Public, name)
	return idx
}

func (system *System) AddSecretVariable(name string) (idx int) {
	idx = system.GetNbSecretVariables() + system.GetNbPublicVariables()
	system.Secret = append(system.Secret, name)
	return idx
}

func (system *System) AddSolverHint(f solver.Hint, id solver.HintID, input []LinearExpression, nbOutput int) (internalVariables []int, err error) {
	if nbOutput <= 0 {
		return nil, fmt.Errorf("hint function must return at least one output")
	}

	var name string
	if id == solver.GetHintID(f) {
		name = solver.GetHintName(f)
	} else {
		name = strconv.Itoa(int(id))
	}

	// register the hint as dependency
	if registeredName, ok := system.MHintsDependencies[id]; ok {
		// hint already registered, let's ensure string registeredName matches
		if registeredName != name {
			return nil, fmt.Errorf("hint dependency registration failed; %s previously register with same UUID as %s", name, registeredName)
		}
	} else {
		system.MHintsDependencies[id] = name
	}

	// prepare wires
	internalVariables = make([]int, nbOutput)
	for i := 0; i < len(internalVariables); i++ {
		internalVariables[i] = system.AddInternalVariable()
	}

	// associate these wires with the solver hint
	hm := HintMapping{
		HintID: id,
		Inputs: input,
		OutputRange: struct {
			Start uint32
			End   uint32
		}{
			uint32(internalVariables[0]),
			uint32(internalVariables[len(internalVariables)-1]) + 1,
		},
	}

	blueprint := system.Blueprints[system.genericHint]

	// get []uint32 from the pool
	calldata := getBuffer()

	blueprint.(BlueprintHint).CompressHint(hm, calldata)

	system.AddInstruction(system.genericHint, *calldata)

	// return []uint32 to the pool
	putBuffer(calldata)

	return
}

func (system *System) AddCommitment(c Commitment) error {
	switch v := c.(type) {
	case Groth16Commitment:
		system.CommitmentInfo = append(system.CommitmentInfo.(Groth16Commitments), v)
	case PlonkCommitment:
		system.CommitmentInfo = append(system.CommitmentInfo.(PlonkCommitments), v)
	default:
		return fmt.Errorf("unknown commitment type %T", v)
	}
	return nil
}

func (system *System) AddLog(l LogEntry) {
	system.Logs = append(system.Logs, l)
}

func (system *System) AttachDebugInfo(debugInfo DebugInfo, constraintID []int) {
	system.DebugInfo = append(system.DebugInfo, LogEntry(debugInfo))
	id := len(system.DebugInfo) - 1
	for _, cID := range constraintID {
		system.MDebug[cID] = id
	}
}

// VariableToString implements Resolver
func (system *System) VariableToString(vID int) string {
	nbPublic := system.GetNbPublicVariables()
	nbSecret := system.GetNbSecretVariables()

	if vID < nbPublic {
		return system.Public[vID]
	}
	vID -= nbPublic
	if vID < nbSecret {
		return system.Secret[vID]
	}
	vID -= nbSecret
	return fmt.Sprintf("v%d", vID) // TODO @gbotrel  vs strconv.Itoa.
}

func (cs *System) AddR1C(c R1C, bID BlueprintID) int {
	profile.RecordConstraint()

	blueprint := cs.Blueprints[bID]

	// get a []uint32 from a pool
	calldata := getBuffer()

	// compress the R1C into a []uint32 and add the instruction
	blueprint.(BlueprintR1C).CompressR1C(&c, calldata)
	cs.AddInstruction(bID, *calldata)

	// release the []uint32 to the pool
	putBuffer(calldata)

	return cs.NbConstraints - 1
}

func (cs *System) AddSparseR1C(c SparseR1C, bID BlueprintID) int {
	profile.RecordConstraint()

	blueprint := cs.Blueprints[bID]

	// get a []uint32 from a pool
	calldata := getBuffer()

	// compress the SparceR1C into a []uint32 and add the instruction
	blueprint.(BlueprintSparseR1C).CompressSparseR1C(&c, calldata)

	cs.AddInstruction(bID, *calldata)

	// release the []uint32 to the pool
	putBuffer(calldata)

	return cs.NbConstraints - 1
}

func (cs *System) AddInstruction(bID BlueprintID, calldata []uint32) []uint32 {
	// set the offsets
	pi := PackedInstruction{
		StartCallData:    uint64(len(cs.CallData)),
		ConstraintOffset: uint32(cs.NbConstraints),
		WireOffset:       uint32(cs.NbInternalVariables + cs.GetNbPublicVariables() + cs.GetNbSecretVariables()),
		BlueprintID:      bID,
	}

	// append the call data
	cs.CallData = append(cs.CallData, calldata...)

	// update the total number of constraints
	blueprint := cs.Blueprints[pi.BlueprintID]
	cs.NbConstraints += blueprint.NbConstraints()

	// add the output wires
	inst := pi.Unpack(cs)
	nbOutputs := blueprint.NbOutputs(inst)
	var wires []uint32
	for i := 0; i < nbOutputs; i++ {
		wires = append(wires, uint32(cs.AddInternalVariable()))
	}

	// add the instruction
	cs.Instructions = append(cs.Instructions, pi)

	// update the instruction dependency tree
	level := blueprint.UpdateInstructionTree(inst, cs)
	iID := uint32(len(cs.Instructions) - 1)

	// we can't skip levels, so appending is fine.
	if int(level) >= len(cs.Levels) {
		cs.Levels = append(cs.Levels, []uint32{iID})
	} else {
		cs.Levels[level] = append(cs.Levels[level], iID)
	}

	return wires
}

// GetNbConstraints returns the number of constraints
func (cs *System) GetNbConstraints() int {
	return cs.NbConstraints
}

func (cs *System) CheckUnconstrainedWires() error {
	// TODO @gbotrel
	return nil
}

func (cs *System) GetR1CIterator() R1CIterator {
	return R1CIterator{cs: cs}
}

func (cs *System) GetSparseR1CIterator() SparseR1CIterator {
	return SparseR1CIterator{cs: cs}
}

func (cs *System) GetCommitments() Commitments {
	return cs.CommitmentInfo
}

// bufPool is a pool of buffers used by getBuffer and putBuffer.
// It is used to avoid allocating buffers for each constraint.
var bufPool = sync.Pool{
	New: func() interface{} {
		r := make([]uint32, 0, 20)
		return &r
	},
}

// getBuffer returns a buffer of at least the given size.
// The buffer is taken from the pool if it is large enough,
// otherwise a new buffer is allocated.
// Caller must call putBuffer when done with the buffer.
func getBuffer() *[]uint32 {
	to := bufPool.Get().(*[]uint32)
	*to = (*to)[:0]
	return to
}

// putBuffer returns a buffer to the pool.
func putBuffer(buf *[]uint32) {
	if buf == nil {
		panic("invalid entry in putBuffer")
	}
	bufPool.Put(buf)
}

func (system *System) AddGkr(gkr GkrInfo) error {
	if system.GkrInfo.Is() {
		return fmt.Errorf("currently only one GKR sub-circuit per SNARK is supported")
	}

	system.GkrInfo = gkr
	return nil
}

package constraint

import (
	"fmt"
	"io"
	"math/big"

	"github.com/blang/semver/v4"
	"github.com/consensys/gnark"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/internal/tinyfield"
	"github.com/consensys/gnark/internal/utils"
	"github.com/consensys/gnark/logger"
)

// ConstraintSystem interface that all constraint systems implement.
type ConstraintSystem interface {
	io.WriterTo
	io.ReaderFrom
	CoeffEngine

	// IsSolved returns nil if given witness solves the constraint system and error otherwise
	// Deprecated: use _, err := Solve(...) instead
	IsSolved(witness witness.Witness, opts ...solver.Option) error

	// Solve attempts to solves the constraint system using provided witness.
	// Returns an error if the witness does not allow all the constraints to be satisfied.
	// Returns a typed solution (R1CSSolution or SparseR1CSSolution) and nil otherwise.
	Solve(witness witness.Witness, opts ...solver.Option) (any, error)

	// GetNbVariables return number of internal, secret and public Variables
	// Deprecated: use GetNbSecretVariables() instead
	GetNbVariables() (internal, secret, public int)

	GetNbInternalVariables() int
	GetNbSecretVariables() int
	GetNbPublicVariables() int

	GetNbConstraints() int
	GetNbCoefficients() int

	Field() *big.Int
	FieldBitLen() int

	AddPublicVariable(name string) int
	AddSecretVariable(name string) int
	AddInternalVariable() int

	// AddSolverHint adds a hint to the solver such that the output variables will be computed
	// using a call to output := f(input...) at solve time.
	AddSolverHint(f solver.Hint, input []LinearExpression, nbOutput int) (internalVariables []int, err error)

	AddCommitment(c Commitment) error

	AddLog(l LogEntry)

	// MakeTerm returns a new Term. The constraint system may store coefficients in a map, so
	// calls to this function will grow the memory usage of the constraint system.
	MakeTerm(coeff *Coeff, variableID int) Term

	NewDebugInfo(errName string, i ...interface{}) DebugInfo

	// AttachDebugInfo enables attaching debug information to multiple constraints.
	// This is more efficient than using the AddConstraint(.., debugInfo) since it will store the
	// debug information only once.
	AttachDebugInfo(debugInfo DebugInfo, constraintID []int)

	// CheckUnconstrainedWires returns and error if the constraint system has wires that are not uniquely constrained.
	// This is experimental.
	CheckUnconstrainedWires() error
}

type Iterable interface {
	// WireIterator returns a new iterator to iterate over the wires of the implementer (usually, a constraint)
	// Call to next() returns the next wireID of the Iterable object and -1 when iteration is over.
	//
	// For example a R1C constraint with L, R, O linear expressions, each of size 2, calling several times
	// 		next := r1c.WireIterator();
	// 		for wID := next(); wID != -1; wID = next() {}
	//		// will return in order L[0],L[1],R[0],R[1],O[0],O[1],-1
	WireIterator() (next func() int)
}

var _ Iterable = &SparseR1C{}
var _ Iterable = &R1C{}

// System contains core elements for a constraint System
type System struct {
	// serialization header
	GnarkVersion string
	ScalarField  string

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

	HintMappings       []HintMapping
	MHints             map[int]int              // maps wireID to hint
	MHintsDependencies map[solver.HintID]string // maps hintID to hint string identifier

	// each level contains independent constraints and can be parallelized
	// it is guaranteed that all dependencies for constraints in a level l are solved
	// in previous levels
	// TODO @gbotrel these are currently updated after we add a constraint.
	// but in case the object is built from a serialized representation
	// we need to init the level builder lbWireLevel from the existing constraints.
	Levels [][]int

	// scalar field
	q      *big.Int `cbor:"-"`
	bitLen int      `cbor:"-"`

	// level builder
	lbWireLevel []int            `cbor:"-"` // at which level we solve a wire. init at -1.
	lbOutputs   []uint32         `cbor:"-"` // wire outputs for current constraint.
	lbHints     map[int]struct{} `cbor:"-"` // hints we processed in current round

	CommitmentInfo Commitment
}

// NewSystem initialize the common structure among constraint system
func NewSystem(scalarField *big.Int) System {
	return System{
		SymbolTable:        debug.NewSymbolTable(),
		MDebug:             map[int]int{},
		GnarkVersion:       gnark.Version.String(),
		ScalarField:        scalarField.Text(16),
		MHints:             make(map[int]int),
		MHintsDependencies: make(map[solver.HintID]string),
		q:                  new(big.Int).Set(scalarField),
		bitLen:             scalarField.BitLen(),
		lbHints:            map[int]struct{}{},
	}
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
		log.Warn().Str("binary", binaryVersion.String()).Str("object", objectVersion.String()).Msg("gnark version (binary) mismatch with constraint system. there are no guarantees on compatibilty")
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

func (system *System) AddSolverHint(f solver.Hint, input []LinearExpression, nbOutput int) (internalVariables []int, err error) {
	if nbOutput <= 0 {
		return nil, fmt.Errorf("hint function must return at least one output")
	}

	// register the hint as dependency
	hintUUID, hintID := solver.GetHintID(f), solver.GetHintName(f)
	if id, ok := system.MHintsDependencies[hintUUID]; ok {
		// hint already registered, let's ensure string id matches
		if id != hintID {
			return nil, fmt.Errorf("hint dependency registration failed; %s previously register with same UUID as %s", hintID, id)
		}
	} else {
		system.MHintsDependencies[hintUUID] = hintID
	}

	// prepare wires
	internalVariables = make([]int, nbOutput)
	for i := 0; i < len(internalVariables); i++ {
		internalVariables[i] = system.AddInternalVariable()
	}

	// associate these wires with the solver hint
	hm := HintMapping{HintID: hintUUID, Inputs: input, Outputs: internalVariables}
	system.HintMappings = append(system.HintMappings, hm)
	n := len(system.HintMappings) - 1
	for _, vID := range internalVariables {
		system.MHints[vID] = n
	}

	return
}

func (system *System) AddCommitment(c Commitment) error {
	if system.CommitmentInfo.Is() {
		return fmt.Errorf("currently only one commitment per circuit is supported")
	}

	system.CommitmentInfo = c

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

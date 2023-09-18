package orm

import (
	"testing"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"
)

func TestSequence(t *testing.T) {
	rapid.Check(t, rapid.Run[*sequenceMachine]())
}

// sequenceMachine is a state machine model of Sequence. It simply uses a uint64
// as the model of the sequence.
type sequenceMachine struct {
	store sdk.KVStore
	seq   *Sequence
	state uint64
}

// Init sets up the real Sequence, including choosing a random initial value,
// and intialises the model state
func (m *sequenceMachine) Init(t *rapid.T) {
	// Create context and KV store
	ctx := NewMockContext()
	m.store = ctx.KVStore(sdk.NewKVStoreKey("test"))

	// Create primary key table
	seq := NewSequence(0x1)
	m.seq = &seq

	// Choose initial sequence value
	initSeqVal := rapid.Uint64().Draw(t, "initSeqVal")
	err := m.seq.InitVal(m.store, initSeqVal)
	require.NoError(t, err)

	// Create model state
	m.state = initSeqVal
}

// Check does nothing, because all our invariants are captured in the commands
func (m *sequenceMachine) Check(t *rapid.T) {}

// NextVal is one of the model commands. It checks that the next value of the
// sequence matches the model and increments the model state.
func (m *sequenceMachine) NextVal(t *rapid.T) {
	// Check that the next value in the sequence matches the model
	require.Equal(t, m.state+1, m.seq.NextVal(m.store))

	// Increment the model state
	m.state++
}

// CurVal is one of the model commands. It checks that the current value of the
// sequence matches the model.
func (m *sequenceMachine) CurVal(t *rapid.T) {
	// Check the current value matches the model
	require.Equal(t, m.state, m.seq.CurVal(m.store))
}

// PeekNextVal is one of the model commands. It checks that the next value of
// the sequence matches the model without modifying the state.
func (m *sequenceMachine) PeekNextVal(t *rapid.T) {
	// Check that the next value in the sequence matches the model
	require.Equal(t, m.state+1, m.seq.PeekNextVal(m.store))
}

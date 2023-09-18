package orm

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func TestPrimaryKeyTable(t *testing.T) {
	rapid.Check(t, rapid.Run[*primaryKeyMachine]())
}

// primaryKeyMachine is a state machine model of the PrimaryKeyTable. The state
// is modelled as a map of strings to TableModels.
type primaryKeyMachine struct {
	store sdk.KVStore
	table *PrimaryKeyTable
	state map[string]*testdata.TableModel
}

// stateKeys gets all the keys in the model map
func (m *primaryKeyMachine) stateKeys() []string {
	keys := make([]string, len(m.state))

	i := 0
	for k := range m.state {
		keys[i] = k
		i++
	}

	return keys
}

// Generate a TableModel that has a 50% chance of being a part of the existing
// state
func (m *primaryKeyMachine) genTableModel() *rapid.Generator[*testdata.TableModel] {
	genStateTableModel := rapid.Custom(func(t *rapid.T) *testdata.TableModel {
		pk := rapid.SampledFrom(m.stateKeys()).Draw(t, "key")
		return m.state[pk]
	})

	if len(m.stateKeys()) == 0 {
		return genTableModel
	} else {
		return rapid.OneOf(genTableModel, genStateTableModel)
	}
}

// Init creates a new instance of the state machine model by building the real
// table and making the empty model map
func (m *primaryKeyMachine) Init(t *rapid.T) {
	// Create context
	ctx := NewMockContext()
	m.store = ctx.KVStore(sdk.NewKVStoreKey("test"))

	// Create primary key table
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)
	table, err := NewPrimaryKeyTable(
		[2]byte{0x1},
		&testdata.TableModel{},
		cdc,
	)
	require.NoError(t, err)

	m.table = table

	// Create model state
	m.state = make(map[string]*testdata.TableModel)
}

// Check that the real values match the state values.
func (m *primaryKeyMachine) Check(t *rapid.T) {
	for i := range m.state {
		has := m.table.Has(m.store, []byte(i))
		require.Equal(t, true, has)
	}
}

// Create is one of the model commands. It adds an object to the table, creating
// an error if it already exists.
func (m *primaryKeyMachine) Create(t *rapid.T) {
	g := genTableModel.Draw(t, "g")
	pk := string(PrimaryKey(g))

	t.Logf("pk: %v", pk)
	t.Logf("m.state: %v", m.state)

	err := m.table.Create(m.store, g)

	if m.state[pk] != nil {
		require.Error(t, err)
	} else {
		require.NoError(t, err)
		m.state[pk] = g
	}
}

// Update is one of the model commands. It updates the value at a given primary
// key and fails if that primary key doesn't already exist in the table.
func (m *primaryKeyMachine) Update(t *rapid.T) {
	tm := m.genTableModel().Draw(t, "tm")

	newName := rapid.StringN(1, 100, 150).Draw(t, "newName")
	tm.Name = newName

	// Perform the real Update
	err := m.table.Update(m.store, tm)

	if m.state[string(PrimaryKey(tm))] == nil {
		// If there's no value in the model, we expect an error
		require.Error(t, err)
	} else {
		// If we have a value in the model, expect no error
		require.NoError(t, err)

		// Update the model with the new value
		m.state[string(PrimaryKey(tm))] = tm
	}
}

// Set is one of the model commands. It sets the value at a key in the table
// whether it exists or not.
func (m *primaryKeyMachine) Set(t *rapid.T) {
	g := genTableModel.Draw(t, "g")
	pk := string(PrimaryKey(g))

	err := m.table.Set(m.store, g)

	require.NoError(t, err)
	m.state[pk] = g
}

// Delete is one of the model commands. It removes the object with the given
// primary key from the table and returns an error if that primary key doesn't
// already exist in the table.
func (m *primaryKeyMachine) Delete(t *rapid.T) {
	tm := m.genTableModel().Draw(t, "tm")

	// Perform the real Delete
	err := m.table.Delete(m.store, tm)

	if m.state[string(PrimaryKey(tm))] == nil {
		// If there's no value in the model, we expect an error
		require.Error(t, err)
	} else {
		// If we have a value in the model, expect no error
		require.NoError(t, err)

		// Delete the value from the model
		delete(m.state, string(PrimaryKey(tm)))
	}
}

// Has is one of the model commands. It checks whether a key already exists in
// the table.
func (m *primaryKeyMachine) Has(t *rapid.T) {
	pk := PrimaryKey(m.genTableModel().Draw(t, "g"))

	realHas := m.table.Has(m.store, pk)
	modelHas := m.state[string(pk)] != nil

	require.Equal(t, realHas, modelHas)
}

// GetOne is one of the model commands. It fetches an object from the table by
// its primary key and returns an error if that primary key isn't in the table.
func (m *primaryKeyMachine) GetOne(t *rapid.T) {
	pk := PrimaryKey(m.genTableModel().Draw(t, "tm"))

	var tm testdata.TableModel

	err := m.table.GetOne(m.store, pk, &tm)
	t.Logf("tm: %v", tm)

	if m.state[string(pk)] == nil {
		require.Error(t, err)
	} else {
		require.NoError(t, err)
		require.Equal(t, *m.state[string(pk)], tm)
	}
}

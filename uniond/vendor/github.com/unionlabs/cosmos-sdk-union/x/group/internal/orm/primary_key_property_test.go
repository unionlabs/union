package orm

import (
	"testing"

	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func TestPrimaryKeyTable(t *testing.T) {
	rapid.Check(t, func(rt *rapid.T) {
		ac := address.NewBech32Codec("cosmos")
		// Init creates a new instance of the state machine model by building the real
		// table and making the empty model map
		// Create context
		key := storetypes.NewKVStoreKey("test")
		testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
		store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

		// Create primary key table
		interfaceRegistry := types.NewInterfaceRegistry()
		cdc := codec.NewProtoCodec(interfaceRegistry)
		table, err := NewPrimaryKeyTable(
			[2]byte{0x1},
			&testdata.TableModel{},
			cdc,
			ac,
		)
		require.NoError(t, err)

		// Create model state
		state := make(map[string]*testdata.TableModel)

		rt.Repeat(map[string]func(*rapid.T){
			// Create is one of the model commands. It adds an object to the table, creating
			// an error if it already exists.
			"Create": func(t *rapid.T) {
				g := genTableModel.Draw(t, "g")
				pk := string(PrimaryKey(g, ac))

				t.Logf("pk: %v", pk)
				t.Logf("state: %v", state)

				err := table.Create(store, g)

				if state[pk] != nil {
					require.Error(t, err)
				} else {
					require.NoError(t, err)
					state[pk] = g
				}
			},

			// Update is one of the model commands. It updates the value at a given primary
			// key and fails if that primary key doesn't already exist in the table.
			"Update": func(t *rapid.T) {
				tm := generateTableModel(state).Draw(t, "tm")

				newName := rapid.StringN(1, 100, 150).Draw(t, "newName")
				tm.Name = newName

				// Perform the real Update
				err := table.Update(store, tm)

				if state[string(PrimaryKey(tm, ac))] == nil {
					// If there's no value in the model, we expect an error
					require.Error(t, err)
				} else {
					// If we have a value in the model, expect no error
					require.NoError(t, err)

					// Update the model with the new value
					state[string(PrimaryKey(tm, ac))] = tm
				}
			},

			// Set is one of the model commands. It sets the value at a key in the table
			// whether it exists or not.
			"Set": func(t *rapid.T) {
				g := genTableModel.Draw(t, "g")
				pk := string(PrimaryKey(g, ac))

				err := table.Set(store, g)

				require.NoError(t, err)
				state[pk] = g
			},

			// Delete is one of the model commands. It removes the object with the given
			// primary key from the table and returns an error if that primary key doesn't
			// already exist in the table.
			"Delete": func(t *rapid.T) {
				tm := generateTableModel(state).Draw(t, "tm")

				// Perform the real Delete
				err := table.Delete(store, tm)

				if state[string(PrimaryKey(tm, ac))] == nil {
					// If there's no value in the model, we expect an error
					require.Error(t, err)
				} else {
					// If we have a value in the model, expect no error
					require.NoError(t, err)

					// Delete the value from the model
					delete(state, string(PrimaryKey(tm, ac)))
				}
			},

			// Has is one of the model commands. It checks whether a key already exists in
			// the table.
			"Has": func(t *rapid.T) {
				pk := PrimaryKey(generateTableModel(state).Draw(t, "g"), ac)

				realHas := table.Has(store, pk)
				modelHas := state[string(pk)] != nil

				require.Equal(t, realHas, modelHas)
			},

			// GetOne is one of the model commands. It fetches an object from the table by
			// its primary key and returns an error if that primary key isn't in the table.
			"GetOne": func(t *rapid.T) {
				pk := PrimaryKey(generateTableModel(state).Draw(t, "tm"), ac)

				var tm testdata.TableModel

				err := table.GetOne(store, pk, &tm)
				t.Logf("tm: %v", tm)

				if state[string(pk)] == nil {
					require.Error(t, err)
				} else {
					require.NoError(t, err)
					require.Equal(t, *state[string(pk)], tm)
				}
			},

			// Check that the real values match the state values.
			"": func(t *rapid.T) {
				for i := range state {
					has := table.Has(store, []byte(i))
					require.Equal(t, true, has)
				}
			},
		})
	})
}

// stateKeys gets all the keys in the model map
func stateKeys(state map[string]*testdata.TableModel) []string {
	keys := make([]string, len(state))

	i := 0
	for k := range state {
		keys[i] = k
		i++
	}

	return keys
}

// generateTableModel a TableModel that has a 50% chance of being a part of the existing
// state
func generateTableModel(state map[string]*testdata.TableModel) *rapid.Generator[*testdata.TableModel] {
	genStateTableModel := rapid.Custom(func(t *rapid.T) *testdata.TableModel {
		pk := rapid.SampledFrom(stateKeys(state)).Draw(t, "key")
		return state[pk]
	})

	if len(stateKeys(state)) == 0 {
		return genTableModel
	}
	return rapid.OneOf(genTableModel, genStateTableModel)
}

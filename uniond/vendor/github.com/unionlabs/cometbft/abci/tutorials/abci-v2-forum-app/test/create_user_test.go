package test

import (
	"encoding/json"
	"testing"

	"github.com/dgraph-io/badger/v4"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/abci/tutorials/abci-v2-forum-app/model"
)

func TestCreateUser(t *testing.T) {
	// Open a temporary database for testing
	opts := badger.DefaultOptions("").WithInMemory(true)
	db, err := badger.Open(opts)
	require.NoError(t, err)
	defer db.Close()

	// Create a new DB instance for testing
	testDB := &model.DB{}
	testDB.Init(db)

	// Create a new user
	user := &model.User{
		Name:      "testuser",
		Moderator: false,
		Banned:    false,
	}

	err = testDB.CreateUser(user)
	require.NoError(t, err)

	// Check that the user was saved to the database
	err = testDB.GetDB().View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte(user.Name))
		if err != nil {
			return err
		}
		var userBytes []byte
		err = item.Value(func(val []byte) error {
			userBytes = append(userBytes, val...)
			return nil
		})
		if err != nil {
			return err
		}
		var savedUser model.User
		err = json.Unmarshal(userBytes, &savedUser)
		if err != nil {
			return err
		}
		assert.Equal(t, user, &savedUser)
		return nil
	})
	require.NoError(t, err)

	// Try to create the user again
	err = testDB.CreateUser(user)
	assert.Error(t, err)

	// Find user by Name
	user2, err := testDB.FindUserByName(user.Name)
	if err != nil {
		t.Fatal(err)
	}
	println("User retrieved is", user2)
}

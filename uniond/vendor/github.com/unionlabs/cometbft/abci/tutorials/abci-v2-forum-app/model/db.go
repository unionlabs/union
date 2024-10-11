package model

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"

	"github.com/dgraph-io/badger/v4"

	"github.com/cometbft/cometbft/abci/types"
)

type DB struct {
	db *badger.DB
}

func (db *DB) Init(database *badger.DB) {
	db.db = database
}

func (db *DB) Commit() error {
	return db.db.Update(func(txn *badger.Txn) error {
		return txn.Commit()
	})
}

func NewDB(dbPath string) (*DB, error) {
	// Open badger DB
	opts := badger.DefaultOptions(dbPath)
	db, err := badger.Open(opts)
	if err != nil {
		return nil, err
	}

	// Create a new DB instance and initialize with badger DB
	dbInstance := &DB{}
	dbInstance.Init(db)

	return dbInstance, nil
}

func (db *DB) GetDB() *badger.DB {
	return db.db
}

func (db *DB) Size() int64 {
	lsm, vlog := db.GetDB().Size()
	return lsm + vlog
}

func (db *DB) CreateUser(user *User) error {
	// Check if the user already exists
	err := db.db.View(func(txn *badger.Txn) error {
		_, err := txn.Get([]byte(user.Name))
		return err
	})
	if err == nil {
		return errors.New("user already exists")
	}

	// Save the user to the database
	err = db.db.Update(func(txn *badger.Txn) error {
		userBytes, err := json.Marshal(user)
		if err != nil {
			return fmt.Errorf("failed to marshal user to JSON: %w", err)
		}
		err = txn.Set([]byte(user.Name), userBytes)
		if err != nil {
			return err
		}
		return nil
	})
	return err
}

func (db *DB) FindUserByName(name string) (*User, error) {
	// Read the user from the database
	var user *User
	err := db.db.View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte(name))
		if err != nil {
			return err
		}
		err = item.Value(func(val []byte) error {
			return json.Unmarshal(val, &user)
		})
		return err
	})
	if err != nil {
		return nil, fmt.Errorf("error in retrieving user: %w", err)
	}
	return user, nil
}

func (db *DB) UpdateOrSetUser(uname string, toBan bool, txn *badger.Txn) error {
	user, err := db.FindUserByName(uname)
	// If user is not in the db, then add it
	if errors.Is(err, badger.ErrKeyNotFound) {
		u := new(User)
		u.Name = uname
		u.Banned = toBan
		user = u
	} else {
		if err != nil {
			return errors.New("not able to process user")
		}
		user.Banned = toBan
	}
	userBytes, err := json.Marshal(user)
	if err != nil {
		return fmt.Errorf("error marshaling user: %w", err)
	}
	return txn.Set([]byte(user.Name), userBytes)
}

func (db *DB) Set(key, value []byte) error {
	return db.db.Update(func(txn *badger.Txn) error {
		return txn.Set(key, value)
	})
}

func ViewDB(db *badger.DB, key []byte) ([]byte, error) {
	var value []byte
	err := db.View(func(txn *badger.Txn) error {
		item, err := txn.Get(key)
		if err != nil {
			if !errors.Is(err, badger.ErrKeyNotFound) {
				return err
			}
			return nil
		}
		value, err = item.ValueCopy(nil)
		return err
	})
	if err != nil {
		return nil, err
	}
	return value, nil
}

func (db *DB) Close() error {
	return db.db.Close()
}

func (db *DB) Get(key []byte) ([]byte, error) {
	return ViewDB(db.db, key)
}

func (db *DB) GetValidators() (validators []types.ValidatorUpdate, err error) {
	err = db.db.View(func(txn *badger.Txn) error {
		opts := badger.DefaultIteratorOptions
		opts.PrefetchSize = 10
		it := txn.NewIterator(opts)
		defer it.Close()
		for it.Rewind(); it.Valid(); it.Next() {
			var err error
			item := it.Item()
			k := item.Key()
			if isValidatorTx(k) {
				err := item.Value(func(v []byte) error {
					validator := new(types.ValidatorUpdate)
					err = types.ReadMessage(bytes.NewBuffer(v), validator)
					if err == nil {
						validators = append(validators, *validator)
					}
					return err
				})
				if err != nil {
					return err
				}
			}
		}
		return nil
	})
	if err != nil {
		return nil, err
	}
	return validators, nil
}

func isValidatorTx(tx []byte) bool {
	return bytes.HasPrefix(tx, []byte("val"))
}

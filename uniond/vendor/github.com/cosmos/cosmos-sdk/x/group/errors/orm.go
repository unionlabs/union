package errors

import "cosmossdk.io/errors"

// mathCodespace is the codespace for all errors defined in orm package
const ormCodespace = "legacy_orm"

var (
	// ErrORMIteratorDone defines an error when an iterator is done
	ErrORMIteratorDone = errors.Register(ormCodespace, 11, "iterator done")

	// ErrORMInvalidIterator defines an error for an invalid iterator
	ErrORMInvalidIterator = errors.Register(ormCodespace, 12, "invalid iterator")

	// ErrORMUniqueConstraint defines an error when a value already exists at a given key
	ErrORMUniqueConstraint = errors.Register(ormCodespace, 13, "unique constraint violation")

	// ErrORMInvalidArgument defines an error when an invalid argument is provided as part of ORM functions
	ErrORMInvalidArgument = errors.Register(ormCodespace, 14, "invalid argument")

	// ErrORMKeyMaxLength defines an error when a key exceeds max length
	ErrORMKeyMaxLength = errors.Register(ormCodespace, 15, "key exceeds max length")

	// ErrORMEmptyKey defines an error for an empty key
	ErrORMEmptyKey = errors.Register(ormCodespace, 47, "cannot use empty key")
)

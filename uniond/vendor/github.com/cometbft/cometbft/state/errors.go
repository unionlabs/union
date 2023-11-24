package state

import (
	"errors"
	"fmt"
)

type (
	ErrInvalidBlock error
	ErrProxyAppConn error

	ErrUnknownBlock struct {
		Height int64
	}

	ErrBlockHashMismatch struct {
		CoreHash []byte
		AppHash  []byte
		Height   int64
	}

	ErrAppBlockHeightTooHigh struct {
		CoreHeight int64
		AppHeight  int64
	}

	ErrAppBlockHeightTooLow struct {
		AppHeight int64
		StoreBase int64
	}

	ErrLastStateMismatch struct {
		Height int64
		Core   []byte
		App    []byte
	}

	ErrStateMismatch struct {
		Got      *State
		Expected *State
	}

	ErrNoValSetForHeight struct {
		Height int64
	}

	ErrNoConsensusParamsForHeight struct {
		Height int64
	}

	ErrNoABCIResponsesForHeight struct {
		Height int64
	}
)

func (e ErrUnknownBlock) Error() string {
	return fmt.Sprintf("could not find block #%d", e.Height)
}

func (e ErrBlockHashMismatch) Error() string {
	return fmt.Sprintf(
		"app block hash (%X) does not match core block hash (%X) for height %d",
		e.AppHash,
		e.CoreHash,
		e.Height,
	)
}

func (e ErrAppBlockHeightTooHigh) Error() string {
	return fmt.Sprintf("app block height (%d) is higher than core (%d)", e.AppHeight, e.CoreHeight)
}

func (e ErrAppBlockHeightTooLow) Error() string {
	return fmt.Sprintf("app block height (%d) is too far below block store base (%d)", e.AppHeight, e.StoreBase)
}

func (e ErrLastStateMismatch) Error() string {
	return fmt.Sprintf(
		"latest CometBFT block (%d) LastAppHash (%X) does not match app's AppHash (%X)",
		e.Height,
		e.Core,
		e.App,
	)
}

func (e ErrStateMismatch) Error() string {
	return fmt.Sprintf(
		"state after replay does not match saved state. Got ----\n%v\nExpected ----\n%v\n",
		e.Got,
		e.Expected,
	)
}

func (e ErrNoValSetForHeight) Error() string {
	return fmt.Sprintf("could not find validator set for height #%d", e.Height)
}

func (e ErrNoConsensusParamsForHeight) Error() string {
	return fmt.Sprintf("could not find consensus params for height #%d", e.Height)
}

func (e ErrNoABCIResponsesForHeight) Error() string {
	return fmt.Sprintf("could not find results for height #%d", e.Height)
}

var ErrFinalizeBlockResponsesNotPersisted = errors.New("node is not persisting finalize block responses")

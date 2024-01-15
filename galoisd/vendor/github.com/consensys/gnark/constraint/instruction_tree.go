package constraint

import (
	"github.com/consensys/gnark/debug"
)

type Level int

const (
	LevelUnset Level = -1
)

type InstructionTree interface {
	// InsertWire inserts a wire in the instruction tree at the given level.
	// If the wire is already in the instruction tree, it panics.
	InsertWire(wire uint32, level Level)

	// HasWire returns true if the wire is in the instruction tree.
	// False if it's a constant or an input.
	HasWire(wire uint32) bool

	// GetWireLevel returns the level of the wire in the instruction tree.
	// If HasWire(wire) returns false, behavior is undefined.
	GetWireLevel(wire uint32) Level
}

// the instruction tree is a simple array of levels.
// it's morally a map[uint32 (wireID)]Level, but we use an array for performance reasons.

func (system *System) HasWire(wireID uint32) bool {
	offset := system.internalWireOffset()
	if wireID < offset {
		// it's a input.
		return false
	}
	// if wireID == maxUint32, it's a constant.
	return (wireID - offset) < uint32(len(system.lbWireLevel))
}

func (system *System) GetWireLevel(wireID uint32) Level {
	return system.lbWireLevel[wireID-system.internalWireOffset()]
}

func (system *System) InsertWire(wireID uint32, level Level) {
	if debug.Debug {
		if level < 0 {
			panic("level must be >= 0")
		}
		if wireID < system.internalWireOffset() {
			panic("cannot insert input wire in instruction tree")
		}
	}
	wireID -= system.internalWireOffset()
	if system.lbWireLevel[wireID] != LevelUnset {
		panic("wire already exist in instruction tree")
	}

	system.lbWireLevel[wireID] = level
}

// internalWireOffset returns the position of the first internal wire in the wireIDs.
func (system *System) internalWireOffset() uint32 {
	return uint32(system.GetNbPublicVariables() + system.GetNbSecretVariables())
}

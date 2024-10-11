// nolint:errcheck
package iavl

import (
	"fmt"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	iavlrand "github.com/cosmos/iavl/internal/rand"
)

// This file implement fuzz testing by generating programs and then running
// them. If an error occurs, the program that had the error is printed.

// A program is a list of instructions.
type program struct {
	instructions []instruction
}

func (p *program) Execute(tree *MutableTree) (err error) {
	var errLine int

	defer func() {
		if r := recover(); r != nil {
			var str string

			for i, instr := range p.instructions {
				prefix := "   "
				if i == errLine {
					prefix = ">> "
				}
				str += prefix + instr.String() + "\n"
			}
			err = fmt.Errorf("program panicked with: %s\n%s", r, str)
		}
	}()

	for i, instr := range p.instructions {
		errLine = i
		instr.Execute(tree)
	}
	return
}

func (p *program) addInstruction(i instruction) {
	p.instructions = append(p.instructions, i)
}

func (p *program) size() int {
	return len(p.instructions)
}

type instruction struct {
	op      string
	k, v    []byte
	version int64
}

func (i instruction) Execute(tree *MutableTree) {
	switch i.op {
	case "SET":
		tree.Set(i.k, i.v)
	case "REMOVE":
		tree.Remove(i.k)
	case "SAVE":
		tree.SaveVersion()
	case "DELETE":
		tree.DeleteVersionsTo(i.version)
	default:
		panic("Unrecognized op: " + i.op)
	}
}

func (i instruction) String() string {
	if i.version > 0 {
		return fmt.Sprintf("%-8s %-8s %-8s %-8d", i.op, i.k, i.v, i.version)
	}
	return fmt.Sprintf("%-8s %-8s %-8s", i.op, i.k, i.v)
}

// Generate a random program of the given size.
func genRandomProgram(size int) *program {
	p := &program{}
	nextVersion := 1

	for p.size() < size {
		k, v := []byte(iavlrand.RandStr(1)), []byte(iavlrand.RandStr(1))

		switch rand.Int() % 7 {
		case 0, 1, 2:
			p.addInstruction(instruction{op: "SET", k: k, v: v})
		case 3, 4:
			p.addInstruction(instruction{op: "REMOVE", k: k})
		case 5:
			p.addInstruction(instruction{op: "SAVE", version: int64(nextVersion)})
			nextVersion++
		case 6:
			if rv := rand.Int() % nextVersion; rv < nextVersion && rv > 0 {
				p.addInstruction(instruction{op: "DELETE", version: int64(rv)})
			}
		}
	}
	return p
}

// Generate many programs and run them.
func TestMutableTreeFuzz(t *testing.T) {
	maxIterations := testFuzzIterations
	progsPerIteration := 100000
	iterations := 0

	for size := 5; iterations < maxIterations; size++ {
		for i := 0; i < progsPerIteration/size; i++ {
			tree := getTestTree(0)
			program := genRandomProgram(size)
			err := program.Execute(tree)
			if err != nil {
				str, err := tree.String()
				require.Nil(t, err)
				t.Fatalf("Error after %d iterations (size %d): %s\n%s", iterations, size, err.Error(), str)
			}
			iterations++
		}
	}
}

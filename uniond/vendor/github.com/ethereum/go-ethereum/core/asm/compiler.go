// Copyright 2017 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package asm

import (
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"os"
	"strings"

	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core/vm"
)

// Compiler contains information about the parsed source
// and holds the tokens for the program.
type Compiler struct {
	tokens []token
	out    []byte

	labels map[string]int

	pc, pos int

	debug bool
}

// NewCompiler returns a new allocated compiler.
func NewCompiler(debug bool) *Compiler {
	return &Compiler{
		labels: make(map[string]int),
		debug:  debug,
	}
}

// Feed feeds tokens into ch and are interpreted by
// the compiler.
//
// feed is the first pass in the compile stage as it collects the used labels in the
// program and keeps a program counter which is used to determine the locations of the
// jump dests. The labels can than be used in the second stage to push labels and
// determine the right position.
func (c *Compiler) Feed(ch <-chan token) {
	var prev token
	for i := range ch {
		switch i.typ {
		case number:
			num := math.MustParseBig256(i.text).Bytes()
			if len(num) == 0 {
				num = []byte{0}
			}
			c.pc += len(num)
		case stringValue:
			c.pc += len(i.text) - 2
		case element:
			c.pc++
		case labelDef:
			c.labels[i.text] = c.pc
			c.pc++
		case label:
			c.pc += 4
			if prev.typ == element && isJump(prev.text) {
				c.pc++
			}
		}
		c.tokens = append(c.tokens, i)
		prev = i
	}
	if c.debug {
		fmt.Fprintln(os.Stderr, "found", len(c.labels), "labels")
	}
}

// Compile compiles the current tokens and returns a binary string that can be interpreted
// by the EVM and an error if it failed.
//
// compile is the second stage in the compile phase which compiles the tokens to EVM
// instructions.
func (c *Compiler) Compile() (string, []error) {
	var errors []error
	// continue looping over the tokens until
	// the stack has been exhausted.
	for c.pos < len(c.tokens) {
		if err := c.compileLine(); err != nil {
			errors = append(errors, err)
		}
	}

	// turn the binary to hex
	h := hex.EncodeToString(c.out)
	return h, errors
}

// next returns the next token and increments the
// position.
func (c *Compiler) next() token {
	token := c.tokens[c.pos]
	c.pos++
	return token
}

// compileLine compiles a single line instruction e.g.
// "push 1", "jump @label".
func (c *Compiler) compileLine() error {
	n := c.next()
	if n.typ != lineStart {
		return compileErr(n, n.typ.String(), lineStart.String())
	}

	lvalue := c.next()
	switch lvalue.typ {
	case eof:
		return nil
	case element:
		if err := c.compileElement(lvalue); err != nil {
			return err
		}
	case labelDef:
		c.compileLabel()
	case lineEnd:
		return nil
	default:
		return compileErr(lvalue, lvalue.text, fmt.Sprintf("%v or %v", labelDef, element))
	}

	if n := c.next(); n.typ != lineEnd {
		return compileErr(n, n.text, lineEnd.String())
	}

	return nil
}

// parseNumber compiles the number to bytes
func parseNumber(tok token) ([]byte, error) {
	if tok.typ != number {
		panic("parseNumber of non-number token")
	}
	num, ok := math.ParseBig256(tok.text)
	if !ok {
		return nil, errors.New("invalid number")
	}
	bytes := num.Bytes()
	if len(bytes) == 0 {
		bytes = []byte{0}
	}
	return bytes, nil
}

// compileElement compiles the element (push & label or both)
// to a binary representation and may error if incorrect statements
// where fed.
func (c *Compiler) compileElement(element token) error {
	switch {
	case isJump(element.text):
		return c.compileJump(element.text)
	case isPush(element.text):
		return c.compilePush()
	default:
		c.outputOpcode(toBinary(element.text))
		return nil
	}
}

func (c *Compiler) compileJump(jumpType string) error {
	rvalue := c.next()
	switch rvalue.typ {
	case number:
		numBytes, err := parseNumber(rvalue)
		if err != nil {
			return err
		}
		c.outputBytes(numBytes)

	case stringValue:
		// strings are quoted, remove them.
		str := rvalue.text[1 : len(rvalue.text)-2]
		c.outputBytes([]byte(str))

	case label:
		c.outputOpcode(vm.PUSH4)
		pos := big.NewInt(int64(c.labels[rvalue.text])).Bytes()
		pos = append(make([]byte, 4-len(pos)), pos...)
		c.outputBytes(pos)

	case lineEnd:
		// push without argument is supported, it just takes the destination from the stack.
		c.pos--

	default:
		return compileErr(rvalue, rvalue.text, "number, string or label")
	}
	// push the operation
	c.outputOpcode(toBinary(jumpType))
	return nil
}

func (c *Compiler) compilePush() error {
	// handle pushes. pushes are read from left to right.
	var value []byte
	rvalue := c.next()
	switch rvalue.typ {
	case number:
		value = math.MustParseBig256(rvalue.text).Bytes()
		if len(value) == 0 {
			value = []byte{0}
		}
	case stringValue:
		value = []byte(rvalue.text[1 : len(rvalue.text)-1])
	case label:
		value = big.NewInt(int64(c.labels[rvalue.text])).Bytes()
		value = append(make([]byte, 4-len(value)), value...)
	default:
		return compileErr(rvalue, rvalue.text, "number, string or label")
	}
	if len(value) > 32 {
		return fmt.Errorf("%d: string or number size > 32 bytes", rvalue.lineno+1)
	}
	c.outputOpcode(vm.OpCode(int(vm.PUSH1) - 1 + len(value)))
	c.outputBytes(value)
	return nil
}

// compileLabel pushes a jumpdest to the binary slice.
func (c *Compiler) compileLabel() {
	c.outputOpcode(vm.JUMPDEST)
}

func (c *Compiler) outputOpcode(op vm.OpCode) {
	if c.debug {
		fmt.Printf("%d: %v\n", len(c.out), op)
	}
	c.out = append(c.out, byte(op))
}

// output pushes the value v to the binary stack.
func (c *Compiler) outputBytes(b []byte) {
	if c.debug {
		fmt.Printf("%d: %x\n", len(c.out), b)
	}
	c.out = append(c.out, b...)
}

// isPush returns whether the string op is either any of
// push(N).
func isPush(op string) bool {
	return strings.EqualFold(op, "PUSH")
}

// isJump returns whether the string op is jump(i)
func isJump(op string) bool {
	return strings.EqualFold(op, "JUMPI") || strings.EqualFold(op, "JUMP")
}

// toBinary converts text to a vm.OpCode
func toBinary(text string) vm.OpCode {
	return vm.StringToOp(strings.ToUpper(text))
}

type compileError struct {
	got  string
	want string

	lineno int
}

func (err compileError) Error() string {
	return fmt.Sprintf("%d: syntax error: unexpected %v, expected %v", err.lineno, err.got, err.want)
}

func compileErr(c token, got, want string) error {
	return compileError{
		got:    got,
		want:   want,
		lineno: c.lineno + 1,
	}
}

// Package calc evaluates simple arithmetic expressions.
package calc

import (
	"bytes"
	"errors"
	"math/big"
)

// associativity of an operator.
type associativity int

// associativity values.
const (
	leftassociative associativity = iota
	rightassociative
)

// operator is a binary operator.
type operator struct {
	precedence    int
	associativity associativity
	apply         func(*big.Int, *big.Int, *big.Int) *big.Int
}

// operators supported by the calculator.
var operators = map[byte]operator{
	'^': {precedence: 3, associativity: rightassociative, apply: func(z, x, y *big.Int) *big.Int { return z.Exp(x, y, nil) }},
	'*': {precedence: 3, associativity: leftassociative, apply: (*big.Int).Mul},
	'/': {precedence: 3, associativity: leftassociative, apply: (*big.Int).Div},
	'+': {precedence: 2, associativity: leftassociative, apply: (*big.Int).Add},
	'-': {precedence: 2, associativity: leftassociative, apply: (*big.Int).Sub},
}

// yard implements the "shunting yard" algorithm.
type yard struct {
	operands  []*big.Int
	operators []operator
}

// operand pushes a new operand x.
func (y *yard) operand(x *big.Int) {
	y.operands = append(y.operands, x)
}

// operator pushes a new operator.
func (y *yard) operator(op operator) error {
	// Pop higher precedence operators.
	for len(y.operators) > 0 {
		top := y.peek()
		if top.precedence < op.precedence || (top.precedence == op.precedence && op.associativity != leftassociative) {
			break
		}
		if err := y.apply(top); err != nil {
			return err
		}
		y.pop()
	}

	// Push operator on the stack.
	y.operators = append(y.operators, op)

	return nil
}

// apply operator to the operand stack.
func (y *yard) apply(op operator) error {
	n := len(y.operands)
	if n < 2 {
		return errors.New("too few operands")
	}

	z := new(big.Int)
	op.apply(z, y.operands[n-2], y.operands[n-1])
	y.operands = append(y.operands[:n-2], z)

	return nil
}

// result finalizes the evaluation and returns the result.
func (y *yard) result() (*big.Int, error) {
	for len(y.operators) > 0 {
		if err := y.apply(y.pop()); err != nil {
			return nil, err
		}
	}
	if len(y.operands) != 1 {
		return nil, errors.New("wrong operand count")
	}
	return y.operands[0], nil
}

// peek returns the operator at the top of the stack.
func (y *yard) peek() operator {
	return y.operators[len(y.operators)-1]
}

// pop removes and returns the operator at the top of the stack.
func (y *yard) pop() operator {
	top := len(y.operators) - 1
	op := y.operators[top]
	y.operators = y.operators[:top]
	return op
}

// Eval evaluates the arithmetic expression.
func Eval(expr string) (*big.Int, error) {
	b := []byte(expr)
	y := &yard{}
	operand := true
	for len(b) > 0 {
		// Skip this character?
		if skip(b[0]) {
			b = b[1:]
			continue
		}

		// Expect an operand.
		if operand {
			x, rest, err := number(b)
			if err != nil {
				return nil, err
			}
			y.operand(x)
			b = rest
			operand = false
			continue
		}

		// Expect an operator.
		op, ok := operators[b[0]]
		if !ok {
			return nil, errors.New("expected operator")
		}
		if err := y.operator(op); err != nil {
			return nil, err
		}
		b = b[1:]
		operand = true
	}
	return y.result()
}

// number parses a number.
func number(b []byte) (*big.Int, []byte, error) {
	// Find the end.
	i := 0
	if len(b) > 0 && b[0] == '-' {
		i++
	}
	isdigit := isdecimal
	switch {
	case bytes.HasPrefix(b[i:], []byte("0b")):
		isdigit = isbinary
		i += 2
	case bytes.HasPrefix(b[i:], []byte("0x")):
		isdigit = ishex
		i += 2
	}
	for ; i < len(b) && isdigit(b[i]); i++ {
	}

	// Parse.
	x, ok := new(big.Int).SetString(string(b[:i]), 0)
	if !ok {
		return nil, nil, errors.New("expected number")
	}

	return x, b[i:], nil
}

// skip reports whether b should be skipped.
func skip(b byte) bool {
	return b == ' '
}

// isdecimal reports whether b is a decimal digit.
func isdecimal(b byte) bool {
	return '0' <= b && b <= '9'
}

// ishex reports whether b is a hex digit.
func ishex(b byte) bool {
	return isdecimal(b) || ('a' <= b && b <= 'f')
}

// isbinary reports whether b is a binary digit.
func isbinary(b byte) bool {
	return b == '0' || b == '1'
}

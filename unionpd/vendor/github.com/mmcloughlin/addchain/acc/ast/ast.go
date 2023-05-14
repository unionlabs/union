// Package ast declares abstract syntax tree types for acc programs.
package ast

// Chain represents a sequence of acc statements for an addition chain
// computation.
type Chain struct {
	Statements []Statement
}

// Statement assigns the result of an expression to a variable.
type Statement struct {
	Name Identifier
	Expr Expr
}

// Operator precedence range.
const (
	LowestPrec  = 0
	HighestPrec = 4
)

// Expr is an expression.
type Expr interface {
	Precedence() int
}

// Operand is an index into an addition chain.
type Operand int

// Precedence of this expression type.
func (Operand) Precedence() int { return HighestPrec }

// Identifier is a variable reference.
type Identifier string

// Precedence of this expression type.
func (Identifier) Precedence() int { return HighestPrec }

// Add expression.
type Add struct {
	X, Y Expr
}

// Precedence of this expression type.
func (Add) Precedence() int { return 1 }

// Shift (repeated doubling) expression.
type Shift struct {
	X Expr
	S uint
}

// Precedence of this expression type.
func (Shift) Precedence() int { return 2 }

// Double expression.
type Double struct {
	X Expr
}

// Precedence of this expression type.
func (Double) Precedence() int { return 3 }

// IsOp reports whether the expression is the result of an operator.
func IsOp(e Expr) bool {
	switch e.(type) {
	case Add, Shift, Double:
		return true
	}
	return false
}

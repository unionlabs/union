package constraint

import (
	"strings"
)

// Resolver allows pretty printing of constraints.
type Resolver interface {
	CoeffToString(coeffID int) string
	VariableToString(variableID int) string
}

// StringBuilder is a helper to build string from constraints, linear expressions or terms.
// It embeds a strings.Builder object for convenience.
type StringBuilder struct {
	strings.Builder
	Resolver
}

// NewStringBuilder returns a new StringBuilder.
func NewStringBuilder(r Resolver) *StringBuilder {
	return &StringBuilder{Resolver: r}
}

// WriteLinearExpression appends the linear expression to the current buffer
func (sbb *StringBuilder) WriteLinearExpression(l LinearExpression) {
	for i := 0; i < len(l); i++ {
		sbb.WriteTerm(l[i])
		if i+1 < len(l) {
			sbb.WriteString(" + ")
		}
	}
}

// WriteLinearExpression appends the term to the current buffer
func (sbb *StringBuilder) WriteTerm(t Term) {
	if t.CoeffID() == CoeffIdZero {
		sbb.WriteByte('0')
		return
	}
	vs := sbb.VariableToString(t.WireID())
	if t.CoeffID() == CoeffIdOne {
		// print the variable only
		sbb.WriteString(vs)
		return
	}
	sbb.WriteString(sbb.CoeffToString(t.CoeffID()))
	if t.WireID() == 0 && vs == "1" {
		// special path for R1CS; the one wire so let's just print the coeff for clarity
		return
	}
	sbb.WriteString("⋅")
	sbb.WriteString(vs)
}

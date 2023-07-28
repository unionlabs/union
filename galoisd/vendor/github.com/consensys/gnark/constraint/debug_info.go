package constraint

import (
	"strings"

	"github.com/consensys/gnark/internal/utils"
)

type DebugInfo LogEntry

func (system *System) NewDebugInfo(errName string, i ...interface{}) DebugInfo {
	var l LogEntry

	const minLogSize = 500
	var sbb strings.Builder
	sbb.Grow(minLogSize)
	sbb.WriteString("[")
	sbb.WriteString(errName)
	sbb.WriteString("] ")

	for _, _i := range i {
		switch v := _i.(type) {
		case LinearExpression:
			l.WriteVariable(v, &sbb)
		case string:
			sbb.WriteString(v)
		case Term:
			l.WriteVariable(LinearExpression{v}, &sbb)
		default:
			_v := utils.FromInterface(v)
			sbb.WriteString(_v.String())
		}
	}
	sbb.WriteByte('\n')
	sbb.WriteString("%s\n") // some space for the stack.
	l.Format = sbb.String()

	// get the stack
	l.Stack = system.SymbolTable.CollectStack()

	return DebugInfo(l)
}

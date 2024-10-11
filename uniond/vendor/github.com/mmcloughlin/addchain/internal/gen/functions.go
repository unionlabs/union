package gen

import (
	"bufio"
	"reflect"
	"strings"

	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/acc/printer"
)

// Function is a function provided to templates.
type Function struct {
	Name        string
	Description string
	Func        interface{}
}

// Signature returns the function signature.
func (f *Function) Signature() string {
	return reflect.ValueOf(f.Func).Type().String()
}

// Functions is the list of functions provided to templates.
var Functions = []*Function{
	{
		Name:        "add",
		Description: "If the input operation is an `ir.Add` then return it, otherwise return `nil`",
		Func: func(op ir.Op) ir.Op {
			if a, ok := op.(ir.Add); ok {
				return a
			}
			return nil
		},
	},
	{
		Name:        "double",
		Description: "If the input operation is an `ir.Double` then return it, otherwise return `nil`",
		Func: func(op ir.Op) ir.Op {
			if d, ok := op.(ir.Double); ok {
				return d
			}
			return nil
		},
	},
	{
		Name:        "shift",
		Description: "If the input operation is an `ir.Shift` then return it, otherwise return `nil`",
		Func: func(op ir.Op) ir.Op {
			if s, ok := op.(ir.Shift); ok {
				return s
			}
			return nil
		},
	},
	{
		Name:        "inc",
		Description: "Increment an integer",
		Func:        func(n int) int { return n + 1 },
	},
	{
		Name:        "format",
		Description: "Formats an addition chain script (`*ast.Chain`) as a string",
		Func:        printer.String,
	},
	{
		Name:        "split",
		Description: "Calls `strings.Split`",
		Func:        strings.Split,
	},
	{
		Name:        "join",
		Description: "Calls `strings.Join`",
		Func:        strings.Join,
	},
	{
		Name:        "lines",
		Description: "Split input string into lines",
		Func: func(s string) []string {
			var lines []string
			scanner := bufio.NewScanner(strings.NewReader(s))
			for scanner.Scan() {
				lines = append(lines, scanner.Text())
			}
			return lines
		},
	},
}

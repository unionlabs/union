package errcheck

import (
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

const commonSrc = `
package p

type Inner struct {}
func (Inner) Method()

type Outer struct {Inner}
type OuterP struct {*Inner}

type InnerInterface interface {
	Method()
}

type OuterInterface interface {InnerInterface}
type MiddleInterfaceStruct struct {OuterInterface}
type OuterInterfaceStruct struct {MiddleInterfaceStruct}

var c = `

type testCase struct {
	selector   string
	expectedOk bool
	expected   []string
}

func TestWalkThroughEmbeddedInterfaces(t *testing.T) {
	cases := []testCase{
		{"Inner{}.Method", false, nil},
		{"(&Inner{}).Method", false, nil},
		{"Outer{}.Method", false, nil},
		{"InnerInterface.Method", true, []string{"test.InnerInterface"}},
		{"OuterInterface.Method", true, []string{"test.OuterInterface", "test.InnerInterface"}},
		{"OuterInterfaceStruct.Method", true, []string{"test.OuterInterface", "test.InnerInterface"}},
	}

	for _, c := range cases {
		fset := token.NewFileSet()
		f, err := parser.ParseFile(fset, "test", commonSrc+c.selector, 0)
		require.NoError(t, err)

		conf := types.Config{}
		info := types.Info{
			Selections: make(map[*ast.SelectorExpr]*types.Selection),
		}
		_, err = conf.Check("test", fset, []*ast.File{f}, &info)
		require.NoError(t, err)
		ast.Inspect(f, func(n ast.Node) bool {
			s, ok := n.(*ast.SelectorExpr)
			if ok {
				selection, ok := info.Selections[s]
				require.Equal(t, true, ok, "No selection!")
				ts, ok := walkThroughEmbeddedInterfaces(selection)
				if ok != c.expectedOk {
					t.Errorf("expected ok %v got %v", c.expectedOk, ok)
					return false
				}
				if !ok {
					return false
				}

				require.Equal(t, len(c.expected), len(ts))
				for i, e := range c.expected {
					assert.Equal(t, e, ts[i].String(), "mismatch at index %d", i)
				}
			}
			return true
		})
	}
}

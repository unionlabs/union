package source

import (
	"fmt"
	"go/ast"
	"go/token"
)

func scanToDeferLine(fileset *token.FileSet, node ast.Node, lineNum int) ast.Node {
	var matchedNode ast.Node
	ast.Inspect(node, func(node ast.Node) bool {
		switch {
		case node == nil || matchedNode != nil:
			return false
		case fileset.Position(node.End()).Line == lineNum:
			if funcLit, ok := node.(*ast.FuncLit); ok {
				matchedNode = funcLit
				return false
			}
		}
		return true
	})
	debug("defer line node: %s", debugFormatNode{matchedNode})
	return matchedNode
}

func guessDefer(node ast.Node) (ast.Node, error) {
	defers := collectDefers(node)
	switch len(defers) {
	case 0:
		return nil, fmt.Errorf("failed to find expression in defer")
	case 1:
		return defers[0].Call, nil
	default:
		return nil, fmt.Errorf(
			"ambiguous call expression: multiple (%d) defers in call block",
			len(defers))
	}
}

func collectDefers(node ast.Node) []*ast.DeferStmt {
	var defers []*ast.DeferStmt
	ast.Inspect(node, func(node ast.Node) bool {
		if d, ok := node.(*ast.DeferStmt); ok {
			defers = append(defers, d)
			debug("defer: %s", debugFormatNode{d})
			return false
		}
		return true
	})
	return defers
}

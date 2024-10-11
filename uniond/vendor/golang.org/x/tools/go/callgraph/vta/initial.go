// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package vta

import (
	"golang.org/x/tools/go/callgraph"
	"golang.org/x/tools/go/callgraph/internal/chautil"
	"golang.org/x/tools/go/ssa"
)

// calleesFunc abstracts call graph in one direction,
// from call sites to callees.
type calleesFunc func(ssa.CallInstruction) []*ssa.Function

// makeCalleesFunc returns an initial call graph for vta as a
// calleesFunc. If c is not nil, returns callees as given by c.
// Otherwise, it returns chautil.LazyCallees over fs.
func makeCalleesFunc(fs map[*ssa.Function]bool, c *callgraph.Graph) calleesFunc {
	if c == nil {
		return chautil.LazyCallees(fs)
	}
	return func(call ssa.CallInstruction) []*ssa.Function {
		node := c.Nodes[call.Parent()]
		if node == nil {
			return nil
		}
		var cs []*ssa.Function
		for _, edge := range node.Out {
			if edge.Site == call {
				cs = append(cs, edge.Callee.Func)
			}
		}
		return cs
	}
}

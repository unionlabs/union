// +build ignore

package main

import (
	. "github.com/mmcloughlin/avo/build"
	. "github.com/mmcloughlin/avo/operand"
)

//go:generate go run asm2.go -out search_amd64.s -stubs stub_search_amd64.go

func main() {
	TEXT("Search", NOSPLIT, "func(xs []uint64, k uint64) int16")
	Doc("Search finds the first idx for which xs[idx] >= k in xs.")
	ptr := Load(Param("xs").Base(), GP64())
	n := Load(Param("xs").Len(), GP64())
	key := Load(Param("k"), GP64())
	retInd := ReturnIndex(0)
	retVal, err := retInd.Resolve()
	if err != nil {
		panic(err)
	}

	Comment("Save n")
	n2 := GP64()
	MOVQ(n, n2)

	Comment("Initialize idx register to zero.")
	idx := GP64()
	XORL(idx.As32(), idx.As32())

	Label("loop")
	m := Mem{Base: ptr, Index: idx, Scale: 8}

	Comment("Unroll1")
	CMPQ(m, key)
	JAE(LabelRef("Found"))

	Comment("Unroll2")
	CMPQ(m.Offset(16), key)
	JAE(LabelRef("Found2"))

	Comment("Unroll3")
	CMPQ(m.Offset(32), key)
	JAE(LabelRef("Found3"))

	Comment("Unroll4")
	CMPQ(m.Offset(48), key)
	JAE(LabelRef("Found4"))

	Comment("plus8")
	ADDQ(Imm(8), idx)
	CMPQ(idx, n)
	JB(LabelRef("loop"))
	JMP(LabelRef("NotFound"))

	Label("Found2")
	ADDL(Imm(2), idx.As32())
	JMP(LabelRef("Found"))

	Label("Found3")
	ADDL(Imm(4), idx.As32())
	JMP(LabelRef("Found"))

	Label("Found4")
	ADDL(Imm(6), idx.As32())

	Label("Found")
	MOVL(idx.As32(), n2.As32()) // n2 is no longer being used

	Label("NotFound")
	MOVL(n2.As32(), idx.As32())
	SHRL(Imm(31), idx.As32())
	ADDL(n2.As32(), idx.As32())
	SHRL(Imm(1), idx.As32())
	MOVL(idx.As32(), retVal.Addr)
	RET()

	Generate()
}

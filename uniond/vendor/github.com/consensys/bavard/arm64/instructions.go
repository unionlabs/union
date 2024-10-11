package arm64

import (
	"fmt"
	"io"
)

type Arm64 struct {
	w            io.Writer
	labelCounter int // TODO: What's this?
}

func NewArm64(w io.Writer) *Arm64 {
	return &Arm64{w: w}
}

func (arm64 *Arm64) LDP(address string, x,y interface{}, comment ...string) {

	arm64.writeOp(comment, "LDP", address, toTuple(x,y))
}

func (arm64 *Arm64) STP(x, y interface{}, address string, comment ...string) {
	arm64.writeOp(comment, "STP", address, toTuple(x,y))
	//arm64.WriteLn(fmt.Sprintf("STP (R%d, R%d), %s", uint64(x), uint64(y), address))
}

func (arm64 *Arm64) ADDS(op1, op2, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "ADDS", op1, op2, dst)
}

func (arm64 *Arm64) ADCS(op1, op2, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "ADCS", op1, op2, dst)
}

func (arm64 *Arm64) ADC(op1, op2, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "ADC", op1, op2, dst)
}

func (arm64 *Arm64) SUBS(subtrahend, minuend, difference interface{}, comment ...string) {
	arm64.writeOp(comment, "SUBS", subtrahend, minuend, difference)
}

func (arm64 *Arm64) SBCS(subtrahend, minuend, difference interface{}, comment ...string) {
	arm64.writeOp(comment, "SBCS", subtrahend, minuend, difference)
}

func (arm64 *Arm64) ORR(op1, op2, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "ORR", op1, op2, dst)
}

func (arm64 *Arm64) MOVD(src, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "MOVD", src, dst)
}

func (arm64 *Arm64) MUL(op1, op2, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "MUL", op1, op2, dst)
}

func (arm64 *Arm64) UMULH(op1, op2, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "UMULH", op1, op2, dst)
}

func (arm64 *Arm64) CSEL(condition string, ifTrue, ifFalse, dst interface{}, comment ...string) {
	arm64.writeOp(comment, "CSEL", condition, ifTrue, ifFalse, dst)
}

func (arm64 *Arm64) TST(a, b interface{}, comment ...string) {
	arm64.writeOp(comment, "TST", a, b)
}

func (arm64 *Arm64) CMP(a, b interface{}, comment ...string) {
	arm64.writeOp(comment, "CMP", a, b)
}

func (arm64 *Arm64) RegisterOffset(r Register, offset int) string {
	return fmt.Sprintf("%d(R%d)", offset, r)
}

func (arm64 *Arm64) GlobalOffset(name string, offset int) string {
	return fmt.Sprintf("%s<>+%d(SB)", name, offset)
}

func toTuple(x, y interface{}) string {
	return fmt.Sprintf("(%s, %s)", Operand(x), Operand(y))
}

//<copy paste> TODO: Super class?

type Label string

func (arm64 *Arm64) LABEL(l Label) {
	arm64.WriteLn(string(l) + ":")
}

func (arm64 *Arm64) RET() {
	arm64.WriteLn("    RET")
}

func (arm64 *Arm64) WriteLn(s string) {
	arm64.Write(s + "\n")
}

func (arm64 *Arm64) Write(s string) {
	arm64.w.Write([]byte(s))
}

func (arm64 *Arm64) Comment(s string) {
	arm64.WriteLn("    // " + s)
}

func (arm64 *Arm64) FnHeader(funcName string, stackSize, argSize int, reserved ...Register) Registers {
	var header string
	if stackSize == 0 {
		header = "TEXT ·%s(SB), NOSPLIT, $%d-%d"
	} else {
		header = "TEXT ·%s(SB), $%d-%d"
	}

	arm64.WriteLn(fmt.Sprintf(header, funcName, stackSize, argSize))
	r := NewRegisters()
	r.Remove(reserved...)
	return r
}

func Operand(i interface{}) string {
	switch t := i.(type) {
	case string:
		return t
	case Register:
		return t.Name()
	case int:
		switch t {
		case 0:
			return "$0"
		case 1:
			return "$1"
		default:
			return fmt.Sprintf("$%#016x", uint64(t))
		}
	case uint64:
		switch t {
		case 0:
			return "$0"
		case 1:
			return "$1"
		default:
			return fmt.Sprintf("$%#016x", t)
		}
	}
	panic("unsupported interface type")
}

func (arm64 *Arm64) writeOp(comments []string, instruction string, r0 interface{}, r ...interface{}) {
	arm64.Write(fmt.Sprintf("    %s %s", instruction, Operand(r0)))
	l := len(Operand(r0))
	for _, rn := range r {
		arm64.Write(fmt.Sprintf(", %s", Operand(rn)))
		l += 2 + len(Operand(rn))
	}
	if len(comments) == 1 {
		l = 50 - l
		for i := 0; i < l; i++ {
			arm64.Write(" ")
		}
		arm64.Write("// " + comments[0])
	}
	arm64.Write("\n")
}

// </ copy paste>
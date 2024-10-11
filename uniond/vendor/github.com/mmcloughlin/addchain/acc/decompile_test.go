package acc

import (
	"reflect"
	"testing"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/acc/pass"
	"github.com/mmcloughlin/addchain/internal/assert"
)

func TestDecompileExample(t *testing.T) {
	p := addchain.Program{}
	_, err := p.Double(0)
	assert.NoError(t, err)
	_, err = p.Add(0, 1)
	assert.NoError(t, err)
	_, err = p.Shift(1, 3)
	assert.NoError(t, err)
	_, err = p.Add(0, 5)
	assert.NoError(t, err)

	t.Log(p)

	expect := &ir.Program{
		Instructions: []*ir.Instruction{
			{
				Output: ir.Index(1),
				Op: ir.Double{
					X: ir.Index(0),
				},
			},
			{
				Output: ir.Index(2),
				Op: ir.Add{
					X: ir.Index(0),
					Y: ir.Index(1),
				},
			},
			{
				Output: ir.Index(5),
				Op: ir.Shift{
					X: ir.Index(1),
					S: 3,
				},
			},
			{
				Output: ir.Index(6),
				Op: ir.Add{
					X: ir.Index(0),
					Y: ir.Index(5),
				},
			},
		},
	}

	got, err := Decompile(p)
	if err != nil {
		t.Fatal()
	}

	t.Logf("got:\n%s", got)

	if !reflect.DeepEqual(got, expect) {
		t.Logf("expect:\n%s", expect)
		t.Fatal("mismatch")
	}
}

func TestDecompileRandom(t *testing.T) {
	CheckRandom(t, func(t *testing.T, p addchain.Program) {
		r, err := Decompile(p)
		if err != nil {
			t.Fatal(err)
		}

		if err := pass.Validate(r); err != nil {
			t.Fatal(err)
		}
	})
}

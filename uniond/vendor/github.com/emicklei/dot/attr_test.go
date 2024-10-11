package dot

import "testing"

func TestAttributesMap_Attrs(t *testing.T) {
	g := NewGraph()
	g.Attrs("l", "v", "l2", "v2")
	if got, want := g.attributes["l"], "v"; got != want {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}
	if got, want := g.attributes["l2"], "v2"; got != want {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}
}

func TestAttributesMap_AttrsMissingValue(t *testing.T) {
	caught := false
	defer func() {
		if r := recover(); r != nil {
			caught = true
		}
	}()
	NewGraph().Attrs("l", "v", "l2")
	if !caught {
		t.Fail()
	}
}

func TestAttributesMap_EmptyKey_NilValue(t *testing.T) {
	g := NewGraph()
	g.Attr("", "skip")
	novalue := interface{}(nil)
	if got, want := g.Value(""), novalue; got != want {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}
}

package metrics

import (
	"testing"
)

func TestGaugeInfoJsonString(t *testing.T) {
	g := NewGaugeInfo()
	g.Update(GaugeInfoValue{
		"chain_id":   "5",
		"anotherKey": "any_string_value",
		"third_key":  "anything",
	},
	)
	want := `{"anotherKey":"any_string_value","chain_id":"5","third_key":"anything"}`

	original := g.Snapshot()
	g.Update(GaugeInfoValue{"value": "updated"})

	if have := original.Value().String(); have != want {
		t.Errorf("\nhave: %v\nwant: %v\n", have, want)
	}
	if have, want := g.Snapshot().Value().String(), `{"value":"updated"}`; have != want {
		t.Errorf("\nhave: %v\nwant: %v\n", have, want)
	}
}

func TestGetOrRegisterGaugeInfo(t *testing.T) {
	r := NewRegistry()
	NewRegisteredGaugeInfo("foo", r).Update(
		GaugeInfoValue{"chain_id": "5"})
	g := GetOrRegisterGaugeInfo("foo", r).Snapshot()
	if have, want := g.Value().String(), `{"chain_id":"5"}`; have != want {
		t.Errorf("have\n%v\nwant\n%v\n", have, want)
	}
}

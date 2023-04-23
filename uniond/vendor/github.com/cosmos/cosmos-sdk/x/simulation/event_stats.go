package simulation

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
)

// EventStats defines an object that keeps a tally of each event that has occurred
// during a simulation.
type EventStats map[string]map[string]map[string]int

// NewEventStats creates a new empty EventStats object
func NewEventStats() EventStats {
	return make(EventStats)
}

// Tally increases the count of a simulation event.
func (es EventStats) Tally(route, op, evResult string) {
	_, ok := es[route]
	if !ok {
		es[route] = make(map[string]map[string]int)
	}

	_, ok = es[route][op]
	if !ok {
		es[route][op] = make(map[string]int)
	}

	es[route][op][evResult]++
}

// Print the event stats in JSON format.
func (es EventStats) Print(w io.Writer) {
	obj, err := json.MarshalIndent(es, "", " ")
	if err != nil {
		panic(err)
	}

	fmt.Fprintln(w, string(obj))
}

// ExportJSON saves the event stats as a JSON file on a given path
func (es EventStats) ExportJSON(path string) {
	bz, err := json.MarshalIndent(es, "", " ")
	if err != nil {
		panic(err)
	}

	err = os.WriteFile(path, bz, 0o600)
	if err != nil {
		panic(err)
	}
}

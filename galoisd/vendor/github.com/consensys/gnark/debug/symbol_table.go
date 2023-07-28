package debug

import (
	"path/filepath"
	"runtime"
	"strings"
)

type SymbolTable struct {
	Locations  []Location
	Functions  []Function
	mFunctions map[string]int `cbor:"-"` // frame.File+frame.Function to id in Functions
	mLocations map[uint64]int `cbor:"-"` // frame PC to location id in Locations
}

func NewSymbolTable() SymbolTable {
	return SymbolTable{
		mFunctions: map[string]int{},
		mLocations: map[uint64]int{},
	}
}

func (st *SymbolTable) CollectStack() []int {
	var r []int
	if Debug {
		r = make([]int, 0, 2)
	} else {
		r = make([]int, 0, 5)
	}
	// derived from: https://golang.org/pkg/runtime/#example_Frames
	// we stop when func name == Define as it is where the gnark circuit code should start

	// Ask runtime.Callers for up to 10 pcs
	var pc [20]uintptr
	n := runtime.Callers(4, pc[:])
	if n == 0 {
		// No pcs available. Stop now.
		// This can happen if the first argument to runtime.Callers is large.
		return r
	}
	frames := runtime.CallersFrames(pc[:n]) // pass only valid pcs to runtime.CallersFrames
	cpt := 0
	// Loop to get frames.
	// A fixed number of pcs can expand to an indefinite number of Frames.
	for {
		frame, more := frames.Next()
		fe := strings.Split(frame.Function, "/")
		function := fe[len(fe)-1]

		if !Debug {
			if cpt == 2 {
				// limit stack size to 2 when debug is not set.
				break
			}
			if strings.Contains(function, "runtime.gopanic") {
				continue
			}
			if strings.Contains(function, "frontend.(*constraintSystem)") {
				continue
			}
			if strings.Contains(frame.File, "test/engine.go") {
				continue
			}
			if strings.Contains(frame.File, "gnark/frontend/cs") {
				continue
			}
			frame.File = filepath.Base(frame.File)
		}

		r = append(r, st.locationID(&frame))
		cpt++

		if !more {
			break
		}
		if strings.HasSuffix(function, "Define") {
			break
		}
		if strings.HasSuffix(function, "callDeferred") {
			break
		}
	}
	return r
}

func (st *SymbolTable) locationID(frame *runtime.Frame) int {
	lID, ok := st.mLocations[uint64(frame.PC)]
	if !ok {
		// first let's see if we have the function.
		fID, ok := st.mFunctions[frame.File+frame.Function]
		if !ok {
			fe := strings.Split(frame.Function, "/")
			fName := fe[len(fe)-1]
			f := Function{
				Name:       fName,
				SystemName: frame.Function,
				Filename:   frame.File,
			}

			st.Functions = append(st.Functions, f)
			fID = len(st.Functions) - 1
			st.mFunctions[frame.File+frame.Function] = fID
		}

		l := Location{FunctionID: fID, Line: int64(frame.Line)}

		st.Locations = append(st.Locations, l)
		lID = len(st.Locations) - 1
		st.mLocations[uint64(frame.PC)] = lID
	}

	return lID
}

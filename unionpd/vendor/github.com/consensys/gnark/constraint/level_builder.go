package constraint

// The main idea here is to find a naive clustering of independent constraints that can be solved in parallel.
//
// We know that at each constraint, we will have at most one unsolved wire.
// (a constraint may have no unsolved wire in which case it is a plain check that the constraint hold,
// or it may additionally have some wires that will be solved by solver hints)
//
// We build a graph of dependency; we say that a wire is solved at a level l
// --> l = max(level_of_dependencies(wire)) + 1
func (system *System) updateLevel(cID int, c Iterable) {
	level := -1
	wireIterator := c.WireIterator()

	for wID := wireIterator(); wID != -1; wID = wireIterator() {
		// iterate over all wires of the R1C
		system.processWire(uint32(wID), &level)
	}

	// level =  max(dependencies) + 1
	level++

	// mark output wire with level
	for _, wireID := range system.lbOutputs {
		for int(wireID) >= len(system.lbWireLevel) {
			// we didn't encounter this wire yet, we need to grow b.wireLevels
			system.lbWireLevel = append(system.lbWireLevel, -1)
		}
		system.lbWireLevel[wireID] = level
	}

	// we can't skip levels, so appending is fine.
	if level >= len(system.Levels) {
		system.Levels = append(system.Levels, []int{cID})
	} else {
		system.Levels[level] = append(system.Levels[level], cID)
	}
	// clean the table. NB! Do not remove or move, this is required to make the
	// compilation deterministic.
	system.lbOutputs = system.lbOutputs[:0]
	system.lbHints = map[int]struct{}{}
}

func (system *System) processWire(wireID uint32, maxLevel *int) {
	if wireID < uint32(system.GetNbPublicVariables()+system.GetNbSecretVariables()) {
		return // ignore inputs
	}
	for int(wireID) >= len(system.lbWireLevel) {
		// we didn't encounter this wire yet, we need to grow b.wireLevels
		system.lbWireLevel = append(system.lbWireLevel, -1)
	}
	if system.lbWireLevel[wireID] != -1 {
		// we know how to solve this wire, it's a dependency
		if system.lbWireLevel[wireID] > *maxLevel {
			*maxLevel = system.lbWireLevel[wireID]
		}
		return
	}
	// we don't know how to solve this wire; it's either THE wire we have to solve or a hint.
	if hID, ok := system.MHints[int(wireID)]; ok {
		// check that we didn't process that hint already; performance wise, if many wires in a
		// constraint are the output of the same hint, and input to parent hint are themselves
		// computed with a hint, we can suffer.
		// (nominal case: not too many different hints involved for a single constraint)
		if _, ok := system.lbHints[hID]; ok {
			// skip
			return
		}
		system.lbHints[hID] = struct{}{}
		h := &system.HintMappings[hID]

		for _, hwid := range h.Outputs {
			system.lbOutputs = append(system.lbOutputs, uint32(hwid))
		}
		for _, in := range h.Inputs {
			for _, t := range in {
				if !t.IsConstant() {
					system.processWire(t.VID, maxLevel)
				}
			}
		}

		return
	}

	// it's the missing wire
	system.lbOutputs = append(system.lbOutputs, wireID)
}

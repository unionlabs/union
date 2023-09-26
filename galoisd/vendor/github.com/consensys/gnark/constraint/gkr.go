package constraint

import (
	"fmt"
	"github.com/consensys/gnark-crypto/utils"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/std/utils/algo_utils"
	"sort"
)

type GkrVariable int // Just an alias to hide implementation details. May be more trouble than worth

type InputDependency struct {
	OutputWire     int
	OutputInstance int
	InputInstance  int
}

type GkrWire struct {
	Gate            string // TODO: Change to description
	Inputs          []int
	Dependencies    []InputDependency // nil for input wires
	NbUniqueOutputs int
}

type GkrCircuit []GkrWire

type GkrInfo struct {
	Circuit     GkrCircuit
	MaxNIns     int
	NbInstances int
	HashName    string
	SolveHintID solver.HintID
	ProveHintID solver.HintID
}

type GkrPermutations struct {
	SortedInstances      []int
	SortedWires          []int
	InstancesPermutation []int
	WiresPermutation     []int
}

func (w GkrWire) IsInput() bool {
	return len(w.Inputs) == 0
}

func (w GkrWire) IsOutput() bool {
	return w.NbUniqueOutputs == 0
}

// AssignmentOffsets returns the index of the first value assigned to a wire TODO: Explain clearly
func (d *GkrInfo) AssignmentOffsets() []int {
	c := d.Circuit
	res := make([]int, len(c)+1)
	for i := range c {
		nbExplicitAssignments := 0
		if c[i].IsInput() {
			nbExplicitAssignments = d.NbInstances - len(c[i].Dependencies)
		}
		res[i+1] = res[i] + nbExplicitAssignments
	}
	return res
}

func (d *GkrInfo) NewInputVariable() GkrVariable {
	i := len(d.Circuit)
	d.Circuit = append(d.Circuit, GkrWire{})
	return GkrVariable(i)
}

// Compile sorts the circuit wires, their dependencies and the instances
func (d *GkrInfo) Compile(nbInstances int) (GkrPermutations, error) {

	var p GkrPermutations
	d.NbInstances = nbInstances
	// sort the instances to decide the order in which they are to be solved
	instanceDeps := make([][]int, nbInstances)
	for i := range d.Circuit {
		for _, dep := range d.Circuit[i].Dependencies {
			instanceDeps[dep.InputInstance] = append(instanceDeps[dep.InputInstance], dep.OutputInstance)
		}
	}

	p.SortedInstances, _ = algo_utils.TopologicalSort(instanceDeps)
	p.InstancesPermutation = algo_utils.InvertPermutation(p.SortedInstances)

	// this whole circuit sorting is a bit of a charade. if things are built using an api, there's no way it could NOT already be topologically sorted
	// worth keeping for future-proofing?

	inputs := algo_utils.Map(d.Circuit, func(w GkrWire) []int {
		return w.Inputs
	})

	var uniqueOuts [][]int
	p.SortedWires, uniqueOuts = algo_utils.TopologicalSort(inputs)
	p.WiresPermutation = algo_utils.InvertPermutation(p.SortedWires)
	wirePermutationAt := algo_utils.SliceAt(p.WiresPermutation)
	sorted := make([]GkrWire, len(d.Circuit)) // TODO: Directly manipulate d.Circuit instead
	for newI, oldI := range p.SortedWires {
		oldW := d.Circuit[oldI]

		if !oldW.IsInput() {
			d.MaxNIns = utils.Max(d.MaxNIns, len(oldW.Inputs))
		}

		for j := range oldW.Dependencies {
			dep := &oldW.Dependencies[j]
			dep.OutputWire = p.WiresPermutation[dep.OutputWire]
			dep.InputInstance = p.InstancesPermutation[dep.InputInstance]
			dep.OutputInstance = p.InstancesPermutation[dep.OutputInstance]
		}
		sort.Slice(oldW.Dependencies, func(i, j int) bool {
			return oldW.Dependencies[i].InputInstance < oldW.Dependencies[j].InputInstance
		})
		for i := 1; i < len(oldW.Dependencies); i++ {
			if oldW.Dependencies[i].InputInstance == oldW.Dependencies[i-1].InputInstance {
				return p, fmt.Errorf("an input wire can only have one dependency per instance")
			}
		} // TODO: Check that dependencies and explicit assignments cover all instances

		sorted[newI] = GkrWire{
			Gate:            oldW.Gate,
			Inputs:          algo_utils.Map(oldW.Inputs, wirePermutationAt),
			Dependencies:    oldW.Dependencies,
			NbUniqueOutputs: len(uniqueOuts[oldI]),
		}
	}
	d.Circuit = sorted

	return p, nil
}

func (d *GkrInfo) Is() bool {
	return d.Circuit != nil
}

// Chunks returns intervals of instances that are independent of each other and can be solved in parallel
func (c GkrCircuit) Chunks(nbInstances int) []int {
	res := make([]int, 0, 1)
	lastSeenDependencyI := make([]int, len(c))

	for start, end := 0, 0; start != nbInstances; start = end {
		end = nbInstances
		endWireI := -1
		for wI, w := range c {
			if wDepI := lastSeenDependencyI[wI]; wDepI < len(w.Dependencies) && w.Dependencies[wDepI].InputInstance < end {
				end = w.Dependencies[wDepI].InputInstance
				endWireI = wI
			}
		}
		if endWireI != -1 {
			lastSeenDependencyI[endWireI]++
		}
		res = append(res, end)
	}
	return res
}

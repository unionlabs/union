package depinject_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
	"gotest.tools/v3/golden"

	"cosmossdk.io/depinject"
)

type KVStoreKey struct {
	name string
}

type MsgClientA struct {
	key string
}

type KeeperA struct {
	key  KVStoreKey
	name string
}

type KeeperB struct {
	key        KVStoreKey
	msgClientA MsgClientA
}

type KeeperC struct {
	key        KVStoreKey
	msgClientA MsgClientA
}

type KeeperD struct {
	key KVStoreKey
}

type Handler struct {
	Handle func()
}

func (Handler) IsOnePerModuleType() {}

type Command struct {
	Run func()
}

func (Command) IsManyPerContainerType() {}

func ProvideKVStoreKey(moduleKey depinject.ModuleKey) KVStoreKey {
	return KVStoreKey{name: moduleKey.Name()}
}

func ProvideMsgClientA(key depinject.ModuleKey) MsgClientA {
	return MsgClientA{key.Name()}
}

type ModuleA struct{}

func (ModuleA) Provide(key KVStoreKey, moduleKey depinject.OwnModuleKey) (KeeperA, Handler, Command) {
	return KeeperA{key: key, name: depinject.ModuleKey(moduleKey).Name()}, Handler{}, Command{}
}

type ModuleB struct{}

type BDependencies struct {
	depinject.In

	Key KVStoreKey
	A   MsgClientA
}

type BProvides struct {
	depinject.Out

	KeeperB  KeeperB
	Commands []Command
}

func (ModuleB) Provide(dependencies BDependencies) (BProvides, Handler, error) {
	return BProvides{
		KeeperB: KeeperB{
			key:        dependencies.Key,
			msgClientA: dependencies.A,
		},
		Commands: []Command{{}, {}},
	}, Handler{}, nil
}

type ModuleUnexportedDependency struct{}

func (ModuleUnexportedDependency) Provide(dependencies UnexportedFieldCDependencies) (CProvides, Handler, error) {
	return CProvides{
		KeeperC: KeeperC{
			key:        dependencies.key,
			msgClientA: dependencies.A,
		},
		Commands: []Command{{}, {}},
	}, Handler{}, nil
}

type UnexportedFieldCDependencies struct {
	depinject.In

	key KVStoreKey
	A   MsgClientA
}

type CProvides struct {
	depinject.Out

	KeeperC  KeeperC
	Commands []Command
}

type ModuleUnexportedProvides struct{}

type CDependencies struct {
	depinject.In

	Key KVStoreKey
	A   MsgClientA
}

type UnexportedFieldCProvides struct {
	depinject.Out

	keeperC  KeeperC
	Commands []Command
}

func (ModuleUnexportedProvides) Provide(dependencies CDependencies) (UnexportedFieldCProvides, Handler, error) {
	return UnexportedFieldCProvides{
		keeperC: KeeperC{
			key:        dependencies.Key,
			msgClientA: dependencies.A,
		},
		Commands: []Command{{}, {}},
	}, Handler{}, nil
}

type ModuleD struct{}

type DDependencies struct {
	depinject.In

	Key     KVStoreKey
	KeeperC KeeperC
}

type DProvides struct {
	depinject.Out

	KeeperD  KeeperD
	Commands []Command
}

func (ModuleD) Provide(dependencies DDependencies) (DProvides, Handler, error) {
	return DProvides{
		KeeperD: KeeperD{
			key: dependencies.Key,
		},
		Commands: []Command{{}, {}},
	}, Handler{}, nil
}

func TestUnexportedField(t *testing.T) {
	var (
		handlers map[string]Handler
		commands []Command
		a        KeeperA
		c        KeeperC
		d        KeeperD

		scenarioConfigProvides = depinject.Configs(
			depinject.Provide(ProvideMsgClientA),
			depinject.ProvideInModule("runtime", ProvideKVStoreKey),
			depinject.ProvideInModule("a", ModuleA.Provide),
			depinject.ProvideInModule("c", ModuleUnexportedProvides.Provide),
			depinject.Supply(ModuleA{}, ModuleUnexportedProvides{}),
		)

		scenarioConfigDependency = depinject.Configs(
			depinject.Provide(ProvideMsgClientA),
			depinject.ProvideInModule("runtime", ProvideKVStoreKey),
			depinject.ProvideInModule("a", ModuleA.Provide),
			depinject.ProvideInModule("c", ModuleUnexportedDependency.Provide),
			depinject.Supply(ModuleA{}, ModuleUnexportedDependency{}),
		)

		scenarioConfigProvidesDependency = depinject.Configs(
			depinject.Provide(ProvideMsgClientA),
			depinject.ProvideInModule("runtime", ProvideKVStoreKey),
			depinject.ProvideInModule("a", ModuleA.Provide),
			depinject.ProvideInModule("c", ModuleUnexportedProvides.Provide),
			depinject.ProvideInModule("d", ModuleD.Provide),
			depinject.Supply(ModuleA{}, ModuleUnexportedProvides{}, ModuleD{}),
		)
	)

	require.ErrorContains(t,
		depinject.Inject(
			scenarioConfigProvides,
			&handlers,
			&commands,
			&a,
			&c,
		),
		"depinject.Out struct",
	)

	require.ErrorContains(t,
		depinject.Inject(
			scenarioConfigDependency,
			&handlers,
			&commands,
			&a,
			&c,
		),
		"depinject.In struct",
	)

	require.ErrorContains(t,
		depinject.Inject(
			scenarioConfigProvidesDependency,
			&handlers,
			&commands,
			&a,
			&c,
			&d,
		),
		"depinject.Out struct",
	)
}

var scenarioConfig = depinject.Configs(
	depinject.Provide(ProvideMsgClientA),
	depinject.ProvideInModule("runtime", ProvideKVStoreKey),
	depinject.ProvideInModule("a", ModuleA.Provide),
	depinject.ProvideInModule("b", ModuleB.Provide),
	depinject.Supply(ModuleA{}, ModuleB{}),
)

func TestScenario(t *testing.T) {
	var (
		handlers map[string]Handler
		commands []Command
		a        KeeperA
		b        KeeperB
	)
	require.NoError(t,
		depinject.Inject(
			scenarioConfig,
			&handlers,
			&commands,
			&a,
			&b,
		))

	require.Len(t, handlers, 2)
	require.Equal(t, Handler{}, handlers["a"])
	require.Equal(t, Handler{}, handlers["b"])
	require.Len(t, commands, 3)
	require.Equal(t, KeeperA{
		key:  KVStoreKey{name: "a"},
		name: "a",
	}, a)
	require.Equal(t, KeeperB{
		key: KVStoreKey{name: "b"},
		msgClientA: MsgClientA{
			key: "b",
		},
	}, b)
}

func TestResolveError(t *testing.T) {
	var x string
	require.Error(t, depinject.Inject(
		depinject.Provide(
			func(x float64) string { return fmt.Sprintf("%f", x) },
			func(x int) float64 { return float64(x) },
			func(x float32) int { return int(x) },
		),
		&x,
	))
}

func TestCyclic(t *testing.T) {
	var x string
	require.Error(t, depinject.Inject(
		depinject.Provide(
			func(x int) float64 { return float64(x) },
			func(x float64) (int, string) { return int(x), "hi" },
		),
		&x,
	))
}

func TestErrorOption(t *testing.T) {
	err := depinject.Inject(depinject.Error(fmt.Errorf("an error")))
	require.Error(t, err)
}

func TestTrivial(t *testing.T) {
	require.NoError(t, depinject.Inject(depinject.Configs()))
}

func Provide0() int { return 0 }
func Provide1() int { return 1 }

func TestSimple(t *testing.T) {
	var x int
	require.NoError(t,
		depinject.Inject(
			depinject.Provide(Provide1),
			&x,
		),
	)

	require.Error(t,
		depinject.Inject(
			depinject.Provide(Provide0, Provide1),
			&x,
		),
	)
}

func ProvideModuleScoped0(depinject.ModuleKey) int { return 0 }
func ProvideModuleScoped1(depinject.ModuleKey) int { return 1 }
func ProvideFloat64FromInt(x int) float64          { return float64(x) }
func ProvideFloat32FromInt(x int) float32          { return float32(x) }

func TestModuleScoped(t *testing.T) {
	var x int
	require.Error(t,
		depinject.Inject(
			depinject.Provide(
				ProvideModuleScoped0,
			),
			&x,
		),
	)

	var y float64
	require.Error(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Provide(
					ProvideModuleScoped0,
					Provide1,
				),
				depinject.ProvideInModule("a", ProvideFloat64FromInt),
			),
			&y,
		),
	)

	require.Error(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Provide(
					Provide0,
					ProvideModuleScoped0,
				),
				depinject.ProvideInModule("a", ProvideFloat64FromInt),
			),
			&y,
		),
	)

	require.Error(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Provide(
					ProvideModuleScoped0,
					ProvideModuleScoped1,
				),
				depinject.ProvideInModule("a", ProvideFloat64FromInt),
			),
			&y,
		),
	)

	require.NoError(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Provide(ProvideModuleScoped0),
				depinject.ProvideInModule("a", ProvideFloat64FromInt),
			),
			&y,
		),
	)

	require.Error(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Provide(ProvideModuleScoped0),
				depinject.ProvideInModule("", ProvideFloat64FromInt),
			),
			&y,
		),
	)

	var z float32
	require.NoError(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Provide(ProvideModuleScoped0),
				depinject.ProvideInModule("a",
					ProvideFloat64FromInt,
					ProvideFloat32FromInt,
				),
			),
			&y, &z,
		),
		"use module dep twice",
	)
}

type OnePerModuleInt int

func (OnePerModuleInt) IsOnePerModuleType() {}

func OnePerModuleInt3() OnePerModuleInt { return 3 }
func OnePerModuleInt4() OnePerModuleInt { return 4 }
func CollectOnePerModuleInts(x map[string]OnePerModuleInt) string {
	sum := 0
	for _, v := range x {
		sum += int(v)
	}
	return fmt.Sprintf("%d", sum)
}

func ReturnOnePerModuleMap() map[string]OnePerModuleInt { return nil }

func TestOnePerModule(t *testing.T) {
	var x OnePerModuleInt
	require.Error(t,
		depinject.Inject(depinject.Configs(), &x),
		"bad input type",
	)

	var y map[string]OnePerModuleInt
	var z string
	require.NoError(t,
		depinject.Inject(
			depinject.Configs(
				depinject.ProvideInModule("a", OnePerModuleInt3),
				depinject.ProvideInModule("b", OnePerModuleInt4),
				depinject.Provide(CollectOnePerModuleInts),
			),
			&y,
			&z,
		),
	)

	require.Equal(t, map[string]OnePerModuleInt{
		"a": 3,
		"b": 4,
	}, y)
	require.Equal(t, "7", z)

	var m map[string]OnePerModuleInt
	require.Error(t,
		depinject.Inject(
			depinject.ProvideInModule("a",
				OnePerModuleInt3,
				OnePerModuleInt3,
			),
			&m,
		),
		"duplicate",
	)

	require.Error(t,
		depinject.Inject(
			depinject.Provide(
				OnePerModuleInt3,
			),
			&m,
		),
		"out of scope",
	)

	require.Error(t,
		depinject.Inject(
			depinject.Provide(ReturnOnePerModuleMap),
			&m,
		),
		"bad return type",
	)

	require.NoError(t,
		depinject.Inject(depinject.Configs(), &m),
		"no providers",
	)
}

type ManyPerContainerInt int

func (ManyPerContainerInt) IsManyPerContainerType() {}

func ManyPerContainerInt4() ManyPerContainerInt { return 4 }
func ManyPerContainerInt9() ManyPerContainerInt { return 9 }
func CollectManyPerContainerInts(xs []ManyPerContainerInt) string {
	sum := 0
	for _, x := range xs {
		sum += int(x)
	}
	return fmt.Sprintf("%d", sum)
}

func TestManyPerContainer(t *testing.T) {
	var xs []ManyPerContainerInt
	var sum string
	require.NoError(t,
		depinject.Inject(
			depinject.Provide(
				ManyPerContainerInt4, ManyPerContainerInt9,
				CollectManyPerContainerInts,
			),
			&xs,
			&sum,
		),
	)
	require.Len(t, xs, 2)
	require.Contains(t, xs, ManyPerContainerInt(4))
	require.Contains(t, xs, ManyPerContainerInt(9))
	require.Equal(t, "13", sum)

	var z ManyPerContainerInt
	require.Error(t,
		depinject.Inject(depinject.Provide(ManyPerContainerInt4), &z),
		"bad input type",
	)

	require.NoError(t,
		depinject.Inject(
			depinject.Configs(),
			&xs,
		),
		"no providers",
	)
}

func TestSupply(t *testing.T) {
	var x int
	require.NoError(t,
		depinject.Inject(
			depinject.Supply(3),
			&x,
		),
	)
	require.Equal(t, 3, x)

	require.Error(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Supply(3),
				depinject.Provide(func() int { return 4 }),
			),
			&x,
		),
		"can't supply then provide",
	)

	require.Error(t,
		depinject.Inject(
			depinject.Configs(
				depinject.Supply(3),
				depinject.Provide(func() int { return 4 }),
			),
			&x,
		),
		"can't provide then supply",
	)

	require.Error(t,
		depinject.Inject(
			depinject.Supply(3, 4),
			&x,
		),
		"can't supply twice",
	)
}

type TestInput struct {
	depinject.In

	X int `optional:"true"`
	Y float64
}

type TestOutput struct {
	depinject.Out

	X string
	Y int64
}

func ProvideTestOutput() (TestOutput, error) {
	return TestOutput{X: "A", Y: -10}, nil
}

func ProvideTestOutputErr() (TestOutput, error) {
	return TestOutput{}, fmt.Errorf("error")
}

func TestStructArgs(t *testing.T) {
	var input TestInput
	require.Error(t, depinject.Inject(depinject.Configs(), &input))

	require.NoError(t, depinject.Inject(
		depinject.Supply(1.3),
		&input,
	))
	require.Equal(t, 0, input.X)
	require.Equal(t, 1.3, input.Y)

	require.NoError(t, depinject.Inject(
		depinject.Supply(1.3, 1),
		&input,
	))
	require.Equal(t, 1, input.X)
	require.Equal(t, 1.3, input.Y)

	var x string
	var y int64
	require.NoError(t, depinject.Inject(
		depinject.Provide(ProvideTestOutput),
		&x, &y,
	))
	require.Equal(t, "A", x)
	require.Equal(t, int64(-10), y)

	require.Error(t, depinject.Inject(
		depinject.Provide(ProvideTestOutputErr),
		&x,
	))
}

func TestDebugOptions(t *testing.T) {
	var logOut string
	var dotGraph string

	outfile, err := os.CreateTemp("", "out")
	require.NoError(t, err)
	stdout := os.Stdout
	os.Stdout = outfile
	defer func() { os.Stdout = stdout }()
	defer func() {
		err := os.Remove(outfile.Name())
		if err != nil {
			panic(err)
		}
	}()

	graphfile, err := os.CreateTemp("", "graph")
	require.NoError(t, err)
	defer func() {
		err := os.Remove(graphfile.Name())
		if err != nil {
			panic(err)
		}
	}()

	require.NoError(t, depinject.InjectDebug(
		depinject.DebugOptions(
			depinject.Logger(func(s string) {
				logOut += s
			}),
			depinject.Visualizer(func(g string) {
				dotGraph = g
			}),
			depinject.LogVisualizer(),
			depinject.FileVisualizer(graphfile.Name()),
			depinject.StdoutLogger(),
		),
		depinject.Configs(),
	))

	require.Contains(t, logOut, "digraph")
	require.Contains(t, dotGraph, "digraph")

	outfileContents, err := os.ReadFile(outfile.Name())
	require.NoError(t, err)
	require.Contains(t, string(outfileContents), "digraph")

	graphfileContents, err := os.ReadFile(graphfile.Name())
	require.NoError(t, err)
	require.Contains(t, string(graphfileContents), "digraph")
}

func TestGraphAndLogOutput(t *testing.T) {
	var graphOut string
	var b KeeperB
	debugOpts := depinject.DebugOptions(
		depinject.Visualizer(func(dotGraph string) {
			graphOut = dotGraph
		}))
	require.NoError(t, depinject.InjectDebug(debugOpts, scenarioConfig, &b))
	golden.Assert(t, graphOut, "example.dot")

	badConfig := depinject.Configs(
		depinject.ProvideInModule("runtime", ProvideKVStoreKey),
		depinject.ProvideInModule("a", ModuleA.Provide),
		depinject.ProvideInModule("b", ModuleB.Provide),
	)
	require.Error(t, depinject.InjectDebug(debugOpts, badConfig, &b))
	golden.Assert(t, graphOut, "example_error.dot")
}

func TestConditionalDebugging(t *testing.T) {
	logs := ""
	success := false
	conditionalDebugOpt := depinject.DebugOptions(
		depinject.OnError(depinject.Logger(func(s string) {
			logs += s + "\n"
		})),
		depinject.OnSuccess(depinject.DebugCleanup(func() {
			success = true
		})))

	var input TestInput
	require.Error(t, depinject.InjectDebug(
		conditionalDebugOpt,
		depinject.Configs(),
		&input,
	))
	require.Contains(t, logs, `Initializing logger`)
	require.Contains(t, logs, `Registering providers`)
	require.Contains(t, logs, `Registering outputs`)
	require.False(t, success)

	logs = ""
	success = false
	require.NoError(t, depinject.InjectDebug(
		conditionalDebugOpt,
		depinject.Configs(),
	))
	require.Empty(t, logs)
	require.True(t, success)
}

type TestFuncTypesInputs struct {
	depinject.In

	DuckReturner func() Duck `optional:"true"`
}

type smallMallard struct{}

func (smallMallard) quack() {}

func DuckProvider(in TestFuncTypesInputs) Duck {
	if in.DuckReturner != nil {
		return in.DuckReturner()
	}
	return Mallard{}
}

func TestFuncTypes(t *testing.T) {
	var duckReturnerFactory func() Duck
	err := depinject.Inject(
		depinject.Supply(func() Duck { return smallMallard{} }),
		&duckReturnerFactory)
	require.NoError(t, err)
	_, ok := duckReturnerFactory().(smallMallard)
	require.True(t, ok)

	var duck Duck
	err = depinject.Inject(
		depinject.Configs(
			depinject.Supply(func() Duck { return smallMallard{} }),
			depinject.Provide(DuckProvider),
		),
		&duck)
	_, ok = duck.(smallMallard)
	require.True(t, ok)
	require.NoError(t, err)

	err = depinject.Inject(
		depinject.Configs(
			depinject.Provide(DuckProvider),
		),
		&duck)
	_, ok = duck.(Mallard)
	require.True(t, ok)
	require.NoError(t, err)
}

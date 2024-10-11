package depinject_test

import (
	"fmt"
	"reflect"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
)

type Duck interface {
	quack()
}

type (
	Mallard    struct{}
	Canvasback struct{}
	Marbled    struct{}
)

func (duck Mallard) quack()    {}
func (duck Canvasback) quack() {}
func (duck Marbled) quack()    {}

type DuckWrapper struct {
	Module string
	Duck   Duck
}

func (d DuckWrapper) IsManyPerContainerType() {}

type Pond struct {
	Ducks []DuckWrapper
}

func IsResolvedInGlobalScope(t *testing.T, pond Pond, typeName string) {
	t.Helper()

	found := false
	for _, dw := range pond.Ducks {
		if dw.Module == "" {
			require.Contains(t, reflect.TypeOf(dw.Duck).Name(), typeName)
			found = true
		}
	}

	require.True(t, found)
}

func IsResolvedModuleScope(t *testing.T, pond Pond, module, duckType string) {
	t.Helper()

	moduleFound := false
	for _, dw := range pond.Ducks {
		if dw.Module == module {
			require.Contains(t, reflect.TypeOf(dw.Duck).Name(), duckType)
			moduleFound = true
		}
	}
	require.True(t, moduleFound)
}

func ProvideMallard() Mallard { return Mallard{} }

func ProvideCanvasback() Canvasback { return Canvasback{} }

func ProvideMarbled() Marbled { return Marbled{} }

func ProvideDuckWrapper(duck Duck) DuckWrapper {
	return DuckWrapper{Module: "", Duck: duck}
}

func ProvideModuleDuck(duck Duck, key depinject.OwnModuleKey) DuckWrapper {
	return DuckWrapper{Module: depinject.ModuleKey(key).Name(), Duck: duck}
}

func ResolvePond(ducks []DuckWrapper) Pond { return Pond{Ducks: ducks} }

func fullTypeName(typeName string) string {
	return fmt.Sprintf("cosmossdk.io/depinject_test/depinject_test.%s", typeName)
}

func TestProvideNoBinding(t *testing.T) {
	t.Parallel()

	configs := depinject.Configs(
		depinject.Provide(
			ProvideMallard,
			ProvideDuckWrapper,
			ResolvePond,
		),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.NoError(t, err)

	IsResolvedInGlobalScope(t, pond, "Mallard")
}

func TestProvideNoBindingImplementationErrorAmbiguous(t *testing.T) {
	t.Parallel()

	configs := depinject.Configs(
		depinject.Provide(
			ProvideMallard,
			ProvideCanvasback,
			ProvideDuckWrapper,
			ResolvePond,
		),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.ErrorContains(t, err, "Multiple implementations found")
}

func TestBindInterface(t *testing.T) {
	t.Parallel()

	configs := depinject.Configs(
		depinject.BindInterface(fullTypeName("Duck"), fullTypeName("Mallard")),
		depinject.Provide(
			ProvideMallard,
			ProvideDuckWrapper,
			ResolvePond,
		),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.NoError(t, err)
}

func TestBindInterfaceBoundTypeNotProvided(t *testing.T) {
	t.Parallel()

	configs := depinject.Configs(
		depinject.BindInterface(fullTypeName("Duck"), fullTypeName("Marbled")),
		depinject.Provide(
			ProvideMallard,
			ProvideDuckWrapper,
			ResolvePond,
		),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.ErrorContains(t, err, "No type for explicit binding")
}

func TestBindInterfaceOverwriteImplicitTypeResolution(t *testing.T) {
	t.Parallel()

	configs := depinject.Configs(
		depinject.BindInterface(fullTypeName("Duck"), fullTypeName("Marbled")), // overwrite Canvasback
		depinject.Provide(
			ProvideCanvasback,
			ProvideDuckWrapper,
			ResolvePond,
		),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.ErrorContains(t, err, "No type for explicit binding")

	// same in module scope
	moduleName := "A"
	configs = depinject.Configs(
		depinject.BindInterfaceInModule(moduleName, fullTypeName("Duck"), fullTypeName("Marbled")), // overwrite Canvasback
		depinject.Provide(
			ProvideCanvasback,
			ResolvePond,
		),
		depinject.ProvideInModule(moduleName, ProvideModuleDuck),
	)

	err = depinject.Inject(configs, &pond)
	require.ErrorContains(t, err, "No type for explicit binding")
}

func TestBindingInterfaceGlobalScopeApplyToGlobalAndModuleScope(t *testing.T) {
	t.Parallel()

	configs := depinject.Configs(
		depinject.BindInterface(fullTypeName("Duck"), fullTypeName("Mallard")), // order is important
		depinject.Provide(
			ProvideMallard,
			ProvideCanvasback,
			ProvideDuckWrapper,
			ResolvePond,
		),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.NoError(t, err)
	IsResolvedInGlobalScope(t, pond, "Mallard")

	// same in module scope
	moduleName := "A"
	configs = depinject.Configs(
		depinject.BindInterface(fullTypeName("Duck"), fullTypeName("Mallard")),
		depinject.Provide(
			ProvideMallard,
			ProvideCanvasback,
			ResolvePond,
		),
		depinject.ProvideInModule(moduleName, ProvideModuleDuck),
	)

	err = depinject.Inject(configs, &pond)
	require.NoError(t, err)
	IsResolvedModuleScope(t, pond, moduleName, "Mallard")
}

func TestBindingInterfaceModuleScopeApplyOnlyModuleScope(t *testing.T) {
	t.Parallel()

	moduleName := "A"
	configs := depinject.Configs(
		depinject.BindInterfaceInModule(moduleName, fullTypeName("Duck"), fullTypeName("Canvasback")),
		depinject.Provide(
			ProvideMallard,
			ProvideCanvasback,
			ProvideDuckWrapper,
			ResolvePond,
		),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.ErrorContains(t, err, "Multiple implementations found")

	configs = depinject.Configs(
		depinject.BindInterfaceInModule(moduleName, fullTypeName("Duck"), fullTypeName("Canvasback")),
		depinject.Provide(
			ProvideMallard,
			ProvideCanvasback,
			ResolvePond,
		),
		depinject.ProvideInModule(moduleName, ProvideModuleDuck),
	)

	err = depinject.Inject(configs, &pond)
	require.NoError(t, err)
	IsResolvedModuleScope(t, pond, moduleName, "Canvasback")
}

func TestBindingInterfaceModuleScopeApplyCorrectModule(t *testing.T) {
	t.Parallel()

	moduleName := "A"
	configs := depinject.Configs(
		depinject.BindInterfaceInModule(moduleName, fullTypeName("Duck"), fullTypeName("Canvasback")),
		depinject.Provide(
			ProvideMallard,
			ProvideCanvasback,
			ProvideDuckWrapper,
			ResolvePond,
		),
		depinject.ProvideInModule("B", ProvideModuleDuck),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.ErrorContains(t, err, "Multiple implementations found")
}

func TestBindingInterfaceTwoModuleScopedAndGlobalBinding(t *testing.T) {
	t.Parallel()

	moduleA, moduleB, moduleC := "A", "B", "C"

	configs := depinject.Configs(
		depinject.BindInterface(fullTypeName("Duck"), fullTypeName("Marbled")),
		depinject.BindInterfaceInModule(moduleA, fullTypeName("Duck"), fullTypeName("Canvasback")),
		depinject.BindInterfaceInModule(moduleB, fullTypeName("Duck"), fullTypeName("Mallard")),
		depinject.Provide(
			ProvideMallard,
			ProvideCanvasback,
			ProvideMarbled,
			ProvideDuckWrapper,
			ResolvePond,
		),
		depinject.ProvideInModule(moduleA, ProvideModuleDuck),
		depinject.ProvideInModule(moduleB, ProvideModuleDuck),
		depinject.ProvideInModule(moduleC, ProvideModuleDuck),
	)

	var pond Pond
	err := depinject.Inject(configs, &pond)
	require.NoError(t, err)

	IsResolvedModuleScope(t, pond, moduleA, "Canvasback")
	IsResolvedModuleScope(t, pond, moduleB, "Mallard")
	IsResolvedModuleScope(t, pond, moduleC, "Marbled")
	IsResolvedInGlobalScope(t, pond, "Marbled")
}

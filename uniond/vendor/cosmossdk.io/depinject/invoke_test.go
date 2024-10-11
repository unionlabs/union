package depinject_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
)

type InvokeSuite struct {
	i  int
	sp *string
}

func (s *InvokeSuite) IntStringPointerInvoker(i int, sp *string) {
	s.i = i
	s.sp = sp
}

func ProvideLenModuleKey(key depinject.ModuleKey) int {
	return len(key.Name())
}

func IntProvider5() int { return 5 }

func StringPtrProviderFoo() *string {
	x := "foo"
	return &x
}

func TestInvokerNoResolvableDependencies(t *testing.T) {
	t.Parallel()

	invokerSuite := &InvokeSuite{}
	configs := depinject.Configs(
		depinject.Supply(invokerSuite),
		depinject.Invoke((*InvokeSuite).IntStringPointerInvoker),
	)

	err := depinject.Inject(configs)
	require.NoError(t, err)

	// invokers get called even if their dependencies can't be resolved
	// values are still zeroed
	require.Equal(t, 0, invokerSuite.i)
	require.Equal(t, (*string)(nil), invokerSuite.sp)
}

func TestInvokerProvidedDependencies(t *testing.T) {
	t.Parallel()

	invokerSuite := &InvokeSuite{}
	configs := depinject.Configs(
		depinject.Supply(invokerSuite),
		depinject.Provide(IntProvider5, StringPtrProviderFoo),
		depinject.Invoke((*InvokeSuite).IntStringPointerInvoker),
	)

	err := depinject.Inject(configs)
	require.NoError(t, err)

	require.Equal(t, 5, invokerSuite.i)
	require.Equal(t, "foo", *invokerSuite.sp)
}

func TestInvokerScopedDependencies(t *testing.T) {
	t.Parallel()

	moduleName := "test"

	invokerSuite := &InvokeSuite{}
	configs := depinject.Configs(
		depinject.Supply(invokerSuite),
		depinject.Provide(ProvideLenModuleKey),
		depinject.InvokeInModule(moduleName, (*InvokeSuite).IntStringPointerInvoker),
	)

	err := depinject.Inject(configs)
	require.NoError(t, err)

	require.Equal(t, len(moduleName), invokerSuite.i)
	require.Equal(t, (*string)(nil), invokerSuite.sp)
}

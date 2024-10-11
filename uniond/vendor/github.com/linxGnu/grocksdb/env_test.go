package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestEnv(t *testing.T) {
	t.Parallel()

	env := NewDefaultEnv()
	defer env.Destroy()

	env.SetBackgroundThreads(2)
	require.Equal(t, 2, env.GetBackgroundThreads())

	env.SetHighPriorityBackgroundThreads(5)
	require.Equal(t, 5, env.GetHighPriorityBackgroundThreads())

	env.SetLowPriorityBackgroundThreads(6)
	require.Equal(t, 6, env.GetLowPriorityBackgroundThreads())

	env.SetBottomPriorityBackgroundThreads(14)
	require.Equal(t, 14, env.GetBottomPriorityBackgroundThreads())

	env.JoinAllThreads()
	env.LowerHighPriorityThreadPoolCPUPriority()
	env.LowerHighPriorityThreadPoolIOPriority()
	env.LowerThreadPoolCPUPriority()
	env.LowerThreadPoolIOPriority()
}

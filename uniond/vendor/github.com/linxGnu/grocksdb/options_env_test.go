package grocksdb

import "testing"

func TestOptEnv(t *testing.T) {
	t.Parallel()

	opt := NewDefaultEnvOptions()
	defer opt.Destroy()
}

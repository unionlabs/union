package grocksdb

import "testing"

func TestRestoreOption(t *testing.T) {
	t.Parallel()

	ro := NewRestoreOptions()
	defer ro.Destroy()

	ro.SetKeepLogFiles(123)
}

package meta

import (
	"testing"
)

func TestMetaReleaseTime(t *testing.T) {
	_, err := Meta.ReleaseTime()
	if err != nil {
		t.Fatal(err)
	}
}

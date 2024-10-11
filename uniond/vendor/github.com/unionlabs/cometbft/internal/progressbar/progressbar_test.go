package progressbar

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestProgressBar(t *testing.T) {
	zero := int64(0)
	hundred := int64(100)

	var bar Bar
	bar.NewOption(zero, hundred)

	require.Equal(t, zero, bar.start)
	require.Equal(t, zero, bar.cur)
	require.Equal(t, hundred, bar.total)
	require.Equal(t, zero, bar.percent)
	require.Equal(t, "█", bar.graph)
	require.Equal(t, "", bar.rate)

	defer bar.Finish()
	for i := zero; i <= hundred; i++ {
		time.Sleep(1 * time.Millisecond)
		bar.Play(i)
	}

	require.Equal(t, zero, bar.start)
	require.Equal(t, hundred, bar.cur)
	require.Equal(t, hundred, bar.total)
	require.Equal(t, hundred, bar.percent)

	var rate string
	for i := zero; i < hundred/2; i++ {
		rate += "█"
	}

	require.Equal(t, rate, bar.rate)
}

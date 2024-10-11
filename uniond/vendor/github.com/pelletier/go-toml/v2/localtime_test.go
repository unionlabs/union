package toml_test

import (
	"testing"
	"time"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/require"
)

func TestLocalDate_AsTime(t *testing.T) {
	d := toml.LocalDate{2021, 6, 8}
	cast := d.AsTime(time.UTC)
	require.Equal(t, time.Date(2021, time.June, 8, 0, 0, 0, 0, time.UTC), cast)
}

func TestLocalDate_String(t *testing.T) {
	d := toml.LocalDate{2021, 6, 8}
	require.Equal(t, "2021-06-08", d.String())
}

func TestLocalDate_MarshalText(t *testing.T) {
	d := toml.LocalDate{2021, 6, 8}
	b, err := d.MarshalText()
	require.NoError(t, err)
	require.Equal(t, []byte("2021-06-08"), b)
}

func TestLocalDate_UnmarshalMarshalText(t *testing.T) {
	d := toml.LocalDate{}
	err := d.UnmarshalText([]byte("2021-06-08"))
	require.NoError(t, err)
	require.Equal(t, toml.LocalDate{2021, 6, 8}, d)

	err = d.UnmarshalText([]byte("what"))
	require.Error(t, err)
}

func TestLocalTime_String(t *testing.T) {
	d := toml.LocalTime{20, 12, 1, 2, 9}
	require.Equal(t, "20:12:01.000000002", d.String())
	d = toml.LocalTime{20, 12, 1, 0, 0}
	require.Equal(t, "20:12:01", d.String())
	d = toml.LocalTime{20, 12, 1, 0, 9}
	require.Equal(t, "20:12:01.000000000", d.String())
	d = toml.LocalTime{20, 12, 1, 100, 0}
	require.Equal(t, "20:12:01.0000001", d.String())
}

func TestLocalTime_MarshalText(t *testing.T) {
	d := toml.LocalTime{20, 12, 1, 2, 9}
	b, err := d.MarshalText()
	require.NoError(t, err)
	require.Equal(t, []byte("20:12:01.000000002"), b)
}

func TestLocalTime_UnmarshalMarshalText(t *testing.T) {
	d := toml.LocalTime{}
	err := d.UnmarshalText([]byte("20:12:01.000000002"))
	require.NoError(t, err)
	require.Equal(t, toml.LocalTime{20, 12, 1, 2, 9}, d)

	err = d.UnmarshalText([]byte("what"))
	require.Error(t, err)

	err = d.UnmarshalText([]byte("20:12:01.000000002 bad"))
	require.Error(t, err)
}

func TestLocalTime_RoundTrip(t *testing.T) {
	var d struct{ A toml.LocalTime }
	err := toml.Unmarshal([]byte("a=20:12:01.500"), &d)
	require.NoError(t, err)
	require.Equal(t, "20:12:01.500", d.A.String())
}

func TestLocalDateTime_AsTime(t *testing.T) {
	d := toml.LocalDateTime{
		toml.LocalDate{2021, 6, 8},
		toml.LocalTime{20, 12, 1, 2, 9},
	}
	cast := d.AsTime(time.UTC)
	require.Equal(t, time.Date(2021, time.June, 8, 20, 12, 1, 2, time.UTC), cast)
}

func TestLocalDateTime_String(t *testing.T) {
	d := toml.LocalDateTime{
		toml.LocalDate{2021, 6, 8},
		toml.LocalTime{20, 12, 1, 2, 9},
	}
	require.Equal(t, "2021-06-08T20:12:01.000000002", d.String())
}

func TestLocalDateTime_MarshalText(t *testing.T) {
	d := toml.LocalDateTime{
		toml.LocalDate{2021, 6, 8},
		toml.LocalTime{20, 12, 1, 2, 9},
	}
	b, err := d.MarshalText()
	require.NoError(t, err)
	require.Equal(t, []byte("2021-06-08T20:12:01.000000002"), b)
}

func TestLocalDateTime_UnmarshalMarshalText(t *testing.T) {
	d := toml.LocalDateTime{}
	err := d.UnmarshalText([]byte("2021-06-08 20:12:01.000000002"))
	require.NoError(t, err)
	require.Equal(t, toml.LocalDateTime{
		toml.LocalDate{2021, 6, 8},
		toml.LocalTime{20, 12, 1, 2, 9},
	}, d)

	err = d.UnmarshalText([]byte("what"))
	require.Error(t, err)

	err = d.UnmarshalText([]byte("2021-06-08 20:12:01.000000002 bad"))
	require.Error(t, err)
}

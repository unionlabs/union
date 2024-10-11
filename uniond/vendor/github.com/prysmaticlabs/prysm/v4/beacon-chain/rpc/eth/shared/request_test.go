package shared

import (
	"math"
	"net/http/httptest"
	"testing"
)

func TestUintFromQuery_BuilderBoostFactor(t *testing.T) {
	type args struct {
		raw string
	}
	tests := []struct {
		name      string
		args      args
		wantRaw   string
		wantValue uint64
		wantOK    bool
	}{
		{
			name: "builder boost factor of 0 returns 0",
			args: args{
				raw: "0",
			},
			wantRaw:   "0",
			wantValue: 0,
			wantOK:    true,
		},
		{
			name:      "builder boost factor of the default, 100 returns 100",
			args:      args{raw: "100"},
			wantRaw:   "100",
			wantValue: 100,
			wantOK:    true,
		},
		{
			name:      "builder boost factor max uint64 returns max uint64",
			args:      args{raw: "18446744073709551615"},
			wantRaw:   "18446744073709551615",
			wantValue: math.MaxUint64,
			wantOK:    true,
		},
		{
			name:      "builder boost factor as a percentage returns error",
			args:      args{raw: "0.30"},
			wantRaw:   "",
			wantValue: 0,
			wantOK:    false,
		},
		{
			name:      "builder boost factor negative int returns error",
			args:      args{raw: "-100"},
			wantRaw:   "",
			wantValue: 0,
			wantOK:    false,
		},
		{
			name:      "builder boost factor max uint64 +1 returns error",
			args:      args{raw: "18446744073709551616"},
			wantRaw:   "",
			wantValue: 0,
			wantOK:    false,
		},
		{
			name:      "builder boost factor of invalid string returns error",
			args:      args{raw: "asdf"},
			wantRaw:   "",
			wantValue: 0,
			wantOK:    false,
		},
		{
			name:      "builder boost factor of number bigger than uint64 string returns error",
			args:      args{raw: "9871398721983721908372190837219837129803721983719283798217390821739081273918273918273918273981273982139812739821"},
			wantRaw:   "",
			wantValue: 0,
			wantOK:    false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			query := "builder_boost_factor"
			bbreq := httptest.NewRequest("GET", "/eth/v3/validator/blocks/{slot}?builder_boost_factor="+tt.args.raw, nil)
			w := httptest.NewRecorder()
			got, got1, got2 := UintFromQuery(w, bbreq, query, false)
			if got != tt.wantRaw {
				t.Errorf("UintFromQuery() got = %v, wantRaw %v", got, tt.wantRaw)
			}
			if got1 != tt.wantValue {
				t.Errorf("UintFromQuery() got1 = %v, wantRaw %v", got1, tt.wantValue)
			}
			if got2 != tt.wantOK {
				t.Errorf("UintFromQuery() got2 = %v, wantRaw %v", got2, tt.wantOK)
			}
		})
	}
}

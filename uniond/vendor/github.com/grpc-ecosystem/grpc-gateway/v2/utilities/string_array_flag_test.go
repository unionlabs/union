package utilities_test

import (
	"flag"
	"reflect"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/utilities"
)

func TestStringArrayFlag(t *testing.T) {
	tests := []struct {
		name  string
		flags []string
		want  string
	}{
		{
			name:  "No Value",
			flags: []string{},
			want:  "",
		},
		{
			name:  "Single Value",
			flags: []string{"--my_flag=1"},
			want:  "1",
		},
		{
			name:  "Repeated Value",
			flags: []string{"--my_flag=1", "--my_flag=2"},
			want:  "1,2",
		},
		{
			name:  "Repeated Same Value",
			flags: []string{"--my_flag=1", "--my_flag=1"},
			want:  "1,1",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			flagSet := flag.NewFlagSet("test", flag.PanicOnError)
			result := utilities.StringArrayFlag(flagSet, "my_flag", "repeated flag")
			if err := flagSet.Parse(tt.flags); err != nil {
				t.Errorf("flagSet.Parse() failed with %v", err)
			}
			if !reflect.DeepEqual(result.String(), tt.want) {
				t.Errorf("StringArrayFlag() = %v, want %v", result.String(), tt.want)
			}
		})
	}
}

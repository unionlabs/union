package testutil

import (
	"strings"
	"testing"

	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

// ResetArgs resets arguments for the command. It is desired to be a helpful function for the cmd.SetArgs
// in the case of calling multiple times in a unit test, as cmd.SetArgs doesn't
// reset the flag value as expected.
//
// **Warning**: this is only compatible with following flag types:
//  1. the implementations of pflag.Value
//  2. the built-in implementations of pflag.SliceValue
//  3. the custom implementations of pflag.SliceValue that are split by comma ","
//
// see https://github.com/spf13/cobra/issues/2079#issuecomment-1870115781 for more detail info
func ResetArgs(t *testing.T, cmd *cobra.Command) {
	t.Helper()
	// if flags haven't been parsed yet, there is no need to reset the args
	if !cmd.Flags().Parsed() {
		return
	}
	// If flags have already been parsed, we should reset the values of flags that haven't been set
	cmd.Flags().Visit(func(pf *pflag.Flag) {
		// if the flag hasn't been changed, there is no need to reset the args
		if !pf.Changed {
			return
		}
		// handle pflag.SliceValue
		if v, ok := pf.Value.(pflag.SliceValue); ok {
			defVal := strings.Trim(pf.DefValue, "[]")
			defSliceVal := make([]string, 0)
			if defVal != "" {
				defSliceVal = strings.Split(defVal, ",")
			}
			if err := v.Replace(defSliceVal); err != nil {
				t.Errorf("error resetting argument <%s> with default value <%+v>: %v", pf.Name, defSliceVal, err)
			}
			return
		}
		// handle pflag.Value
		if err := pf.Value.Set(pf.DefValue); err != nil {
			t.Errorf("error resetting argument <%s> with default value <%s>: %v", pf.Name, pf.DefValue, err)
		}
	})
}

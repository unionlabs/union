package features

import (
	"reflect"

	"github.com/urfave/cli/v2"
)

// ActiveFlags returns all of the flags that are not Hidden.
func ActiveFlags(flags []cli.Flag) []cli.Flag {
	visible := make([]cli.Flag, 0, len(flags))
	for _, flag := range flags {
		field := flagValue(flag).FieldByName("Hidden")
		if !field.IsValid() || !field.Bool() {
			visible = append(visible, flag)
		}
	}
	return visible
}

func flagValue(f cli.Flag) reflect.Value {
	fv := reflect.ValueOf(f)
	for fv.Kind() == reflect.Ptr {
		fv = reflect.Indirect(fv)
	}
	return fv
}

package cmd

import (
	"fmt"

	"github.com/urfave/cli/v2"
	"github.com/urfave/cli/v2/altsrc"
)

// WrapFlags so that they can be loaded from alternative sources.
func WrapFlags(flags []cli.Flag) []cli.Flag {
	wrapped := make([]cli.Flag, 0, len(flags))
	for _, f := range flags {
		switch t := f.(type) {
		case *cli.BoolFlag:
			f = altsrc.NewBoolFlag(t)
		case *cli.DurationFlag:
			f = altsrc.NewDurationFlag(t)
		case *cli.GenericFlag:
			f = altsrc.NewGenericFlag(t)
		case *cli.Float64Flag:
			f = altsrc.NewFloat64Flag(t)
		case *cli.IntFlag:
			f = altsrc.NewIntFlag(t)
		case *cli.StringFlag:
			f = altsrc.NewStringFlag(t)
		case *cli.StringSliceFlag:
			f = altsrc.NewStringSliceFlag(t)
		case *cli.Uint64Flag:
			f = altsrc.NewUint64Flag(t)
		case *cli.UintFlag:
			f = altsrc.NewUintFlag(t)
		case *cli.PathFlag:
			f = altsrc.NewPathFlag(t)
		case *cli.Int64Flag:
			// Int64Flag does not work. See https://github.com/prysmaticlabs/prysm/issues/6478
			panic(fmt.Sprintf("unsupported flag type %T", f))
		case *cli.IntSliceFlag:
			f = altsrc.NewIntSliceFlag(t)
		default:
			panic(fmt.Sprintf("cannot convert type %T", f))
		}
		wrapped = append(wrapped, f)
	}
	return wrapped
}

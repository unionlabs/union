package bitslice

import "fmt"

type opt struct {
	digits  int
	nocheck bool
}

func parseOpts(opts ...Option) (*opt, error) {
	o := new(opt)
	for _, apply := range opts {
		if err := apply(o); err != nil {
			return nil, err
		}
	}
	return o, nil
}

// Option allows to customize the behavior of functions in this package. See
// [WithNbDigits] and [WithUnconstrainedOutputs] for examples.
type Option func(*opt) error

// WithNbDigits sets the bound on the number of digits the input can have. If
// this is not set, then we use standard binary decomposition of the input. If
// it is set and it is less than the width of the native field, then we use
// lookup table based method for bounding the inputs which is more efficient.
func WithNbDigits(nbDigits int) Option {
	return func(o *opt) error {
		if nbDigits < 1 {
			return fmt.Errorf("given number of digits %d smaller than 1", nbDigits)
		}
		o.digits = nbDigits
		return nil
	}
}

// WithUnconstrainedOutputs allows to skip the output decomposition and outputs
// width checks. Can be used when these are performed by the caller.
func WithUnconstrainedOutputs() Option {
	return func(o *opt) error {
		o.nocheck = true
		return nil
	}
}

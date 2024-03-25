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

type Option func(*opt) error

func WithNbDigits(nbDigits int) Option {
	return func(o *opt) error {
		if nbDigits < 1 {
			return fmt.Errorf("given number of digits %d smaller than 1", nbDigits)
		}
		o.digits = nbDigits
		return nil
	}
}

func WithUnconstrainedOutputs() Option {
	return func(o *opt) error {
		o.nocheck = true
		return nil
	}
}

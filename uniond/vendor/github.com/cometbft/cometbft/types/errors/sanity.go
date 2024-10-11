package errors

import "fmt"

type (
	// ErrNegativeField is returned every time some field which should be non-negative turns out negative.
	ErrNegativeField struct {
		Field string
	}

	// ErrRequiredField is returned every time a required field is not provided.
	ErrRequiredField struct {
		Field string
	}

	// ErrInvalidField is returned every time a value does not pass a validity check.
	ErrInvalidField struct {
		Field  string
		Reason string
	}

	// ErrWrongField is returned every time a value does not pass a validaty check, accompanied with error.
	ErrWrongField struct {
		Field string
		Err   error
	}

	// ErrNegativeOrZeroField is returned every time some field which should be positive turns out negative or zero.
	ErrNegativeOrZeroField struct {
		Field string
	}
)

func (e ErrNegativeField) Error() string {
	return e.Field + " can't be negative"
}

func (e ErrRequiredField) Error() string {
	return e.Field + " is required"
}

func (e ErrInvalidField) Error() string {
	return fmt.Sprintf("invalid field %s %s", e.Field, e.Reason)
}

func (e ErrWrongField) Error() string {
	return fmt.Sprintf("wrong %s: %v", e.Field, e.Err)
}

func (e ErrNegativeOrZeroField) Error() string {
	return e.Field + " must be positive"
}

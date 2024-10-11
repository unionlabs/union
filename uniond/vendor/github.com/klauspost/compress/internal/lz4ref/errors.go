package lz4ref

type Error string

func (e Error) Error() string { return string(e) }

const (
	ErrInvalidSourceShortBuffer Error = "lz4: invalid source or destination buffer too short"
)

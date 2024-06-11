package types

func NewParams(verifyingKey []byte) Params {
	return Params{
		VerifyingKey: verifyingKey,
	}
}

func DefaultParams() Params {
	return NewParams([]byte{})
}

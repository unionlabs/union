package testdata

import (
	"cosmossdk.io/core/address"
	"cosmossdk.io/errors"
)

var ErrTest = errors.Register("table_testdata", 2, "test")

func (g TableModel) PrimaryKeyFields(_ address.Codec) ([]interface{}, error) {
	return []interface{}{g.Id}, nil
}

func (g TableModel) ValidateBasic() error {
	if g.Name == "" {
		return errors.Wrap(ErrTest, "name")
	}
	return nil
}

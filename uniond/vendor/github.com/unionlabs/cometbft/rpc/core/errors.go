package core

type ErrInvalidOrderBy struct {
	OrderBy string
}

func (e ErrInvalidOrderBy) Error() string {
	return "invalid order_by: expected `asc`, `desc` or ``, but got " + e.OrderBy
}

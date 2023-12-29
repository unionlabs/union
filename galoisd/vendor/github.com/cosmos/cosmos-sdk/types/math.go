package types

func (ip IntProto) String() string {
	return ip.Int.String()
}

func (dp DecProto) String() string {
	return dp.Dec.String()
}

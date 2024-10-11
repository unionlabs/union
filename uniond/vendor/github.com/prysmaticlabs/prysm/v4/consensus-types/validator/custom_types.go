package validator

import "strconv"

// Uint64 custom uint64 to be unmarshallable
type Uint64 uint64

// UnmarshalJSON custom unmarshal function for json
func (u *Uint64) UnmarshalJSON(bs []byte) error {
	str := string(bs) // Parse plain numbers directly.
	if str == "" {
		*u = Uint64(0)
		return nil
	}
	if len(bs) >= 3 && bs[0] == '"' && bs[len(bs)-1] == '"' {
		// Unwrap the quotes from string numbers.
		str = string(bs[1 : len(bs)-1])
	}
	x, err := strconv.ParseUint(str, 10, 64)
	if err != nil {
		return err
	}
	*u = Uint64(x)
	return nil
}

// UnmarshalYAML custom unmarshal function for yaml
func (u *Uint64) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var str string
	err := unmarshal(&str)
	if err != nil {
		return err
	}
	x, err := strconv.ParseUint(str, 10, 64)
	if err != nil {
		return err
	}
	*u = Uint64(x)

	return nil
}

package fix

type mystring string

func _(x int16) mystring {
	return mystring(x) // want `conversion from int16 to mystring \(string\)...`
}

package fix

func _(x uint64) {
	println(string(x)) // want `conversion from uint64 to string yields...`
}

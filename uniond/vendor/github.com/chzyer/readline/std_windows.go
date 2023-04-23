// +build windows

package readline

func init() {
	Stdin = NewRawReader()
	Stdout = NewANSIWriter(Stdout)
	Stderr = NewANSIWriter(Stderr)
}

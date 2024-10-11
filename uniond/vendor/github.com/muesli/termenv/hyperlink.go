package termenv

// Hyperlink creates a hyperlink using OSC8.
func Hyperlink(link, name string) string {
	return output.Hyperlink(link, name)
}

// Hyperlink creates a hyperlink using OSC8.
func (o *Output) Hyperlink(link, name string) string {
	return OSC + "8;;" + link + ST + name + OSC + "8;;" + ST
}

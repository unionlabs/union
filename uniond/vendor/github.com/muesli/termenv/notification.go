package termenv

// Notify triggers a notification using OSC777.
func Notify(title, body string) {
	output.Notify(title, body)
}

// Notify triggers a notification using OSC777.
func (o *Output) Notify(title, body string) {
	_, _ = o.WriteString(OSC + "777;notify;" + title + ";" + body + ST)
}

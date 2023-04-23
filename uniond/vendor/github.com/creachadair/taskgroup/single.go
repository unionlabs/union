package taskgroup

// A Solo manages a single background goroutine. The task is started when the
// value is first created, and the caller can use the Wait method to block
// until it has exited.
type Solo struct {
	errc chan error
	err  error
}

// Single runs task in a new goroutine. The caller should call Wait to wait for
// the task to return and collect its error.
func Single(task Task) *Solo {
	// N.B. This is closed by Wait.
	errc := make(chan error, 1)
	go func() { errc <- task() }()

	return &Solo{errc: errc}
}

// Wait blocks until the task monitored by s has returne and returns the error
// value it reported.
func (s *Solo) Wait() error {
	if e, ok := <-s.errc; ok {
		// This is the first call to receive a value; update err and close the
		// channel.
		s.err = e
		close(s.errc)
	}
	return s.err
}

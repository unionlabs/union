package metrics

// ResettingSample converts an ordinary sample into one that resets whenever its
// snapshot is retrieved. This will break for multi-monitor systems, but when only
// a single metric is being pushed out, this ensure that low-frequency events don't
// skew th charts indefinitely.
func ResettingSample(sample Sample) Sample {
	return &resettingSample{
		Sample: sample,
	}
}

// resettingSample is a simple wrapper around a sample that resets it upon the
// snapshot retrieval.
type resettingSample struct {
	Sample
}

// Snapshot returns a read-only copy of the sample with the original reset.
func (rs *resettingSample) Snapshot() SampleSnapshot {
	s := rs.Sample.Snapshot()
	rs.Sample.Clear()
	return s
}

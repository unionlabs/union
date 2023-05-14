package types

// SnapshotOptions defines the snapshot strategy used when determining which
// heights are snapshotted for state sync.
type SnapshotOptions struct {
	// Interval defines at which heights the snapshot is taken.
	Interval uint64

	// KeepRecent defines how many snapshots to keep in heights.
	KeepRecent uint32
}

func NewSnapshotOptions(interval uint64, keepRecent uint32) SnapshotOptions {
	return SnapshotOptions{
		Interval:   interval,
		KeepRecent: keepRecent,
	}
}

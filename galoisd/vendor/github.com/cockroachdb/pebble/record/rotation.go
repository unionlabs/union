// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package record

// RotationHelper is a type used to inform the decision of rotating a record log
// file.
//
// The assumption is that multiple records can be coalesced into a single record
// (called a snapshot). Starting a new file, where the first record is a
// snapshot of the current state is referred to as "rotating" the log.
//
// Normally we rotate files when a certain file size is reached. But in certain
// cases (e.g. contents become very large), this can result in too frequent
// rotation. This helper contains logic to impose extra conditions on the
// rotation.
//
// The rotation helper uses "size" as a unit-less estimation that is correlated
// with the on-disk size of a record or snapshot.
type RotationHelper struct {
	// lastSnapshotSize is the size of the last snapshot.
	lastSnapshotSize int64
	// sizeSinceLastSnapshot is the sum of sizes of records applied since the last
	// snapshot.
	sizeSinceLastSnapshot int64
	lastRecordSize        int64
}

// AddRecord makes the rotation helper aware of a new record.
func (rh *RotationHelper) AddRecord(recordSize int64) {
	rh.sizeSinceLastSnapshot += recordSize
	rh.lastRecordSize = recordSize
}

// ShouldRotate returns whether we should start a new log file (with a snapshot).
// Does not need to be called if other rotation factors (log file size) are not
// satisfied.
func (rh *RotationHelper) ShouldRotate(nextSnapshotSize int64) bool {
	// The primary goal is to ensure that when reopening a log file, the number of
	// edits that need to be replayed on top of the snapshot is "sane" while
	// keeping the rotation frequency as low as possible.
	//
	// For the purposes of this description, we assume that the log is mainly
	// storing a collection of "entries", with edits adding or removing entries.
	// Consider the following cases:
	//
	// - The number of live entries is roughly stable: after writing the snapshot
	//   (with S entries), we require that there be enough edits such that the
	//   cumulative number of entries in those edits, E, be greater than S. This
	//   will ensure that at most 50% of data written out is due to rotation.
	//
	// - The number of live entries K in the DB is shrinking drastically, say from
	//   S to S/10: After this shrinking, E = 0.9S, and so if we used the previous
	//   snapshot entry count, S, as the threshold that needs to be exceeded, we
	//   will further delay the snapshot writing. Which means on reopen we will
	//   need to replay 0.9S edits to get to a version with 0.1S entries. It would
	//   be better to create a new snapshot when E exceeds the number of entries in
	//   the current version.
	//
	// - The number of live entries L in the DB is growing; say the last snapshot
	//   had S entries, and now we have 10S entries, so E = 9S. If we required
	//   that E is at least the current number of entries, we would further delay
	//   writing a new snapshot (which is not desirable).
	//
	// The logic below uses the min of the last snapshot size count and the size
	// count in the current version.
	return rh.sizeSinceLastSnapshot > rh.lastSnapshotSize || rh.sizeSinceLastSnapshot > nextSnapshotSize
}

// Rotate makes the rotation helper aware that we are rotating to a new snapshot
// (to which we will apply the latest edit).
func (rh *RotationHelper) Rotate(snapshotSize int64) {
	rh.lastSnapshotSize = snapshotSize
	rh.sizeSinceLastSnapshot = rh.lastRecordSize
}

// DebugInfo returns the last snapshot size and size of the edits since the last
// snapshot; used for testing and debugging.
func (rh *RotationHelper) DebugInfo() (lastSnapshotSize int64, sizeSinceLastSnapshot int64) {
	return rh.lastSnapshotSize, rh.sizeSinceLastSnapshot
}

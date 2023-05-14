// Copyright (c) 2012, Suryandaru Triandana <syndtr@gmail.com>
// All rights reserved.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package leveldb

import (
	"fmt"
	"sync/atomic"
	"time"

	"github.com/syndtr/goleveldb/leveldb/journal"
	"github.com/syndtr/goleveldb/leveldb/storage"
)

// Logging.

type dropper struct {
	s  *session
	fd storage.FileDesc
}

func (d dropper) Drop(err error) {
	if e, ok := err.(*journal.ErrCorrupted); ok {
		d.s.logf("journal@drop %s-%d S·%s %q", d.fd.Type, d.fd.Num, shortenb(e.Size), e.Reason)
	} else {
		d.s.logf("journal@drop %s-%d %q", d.fd.Type, d.fd.Num, err)
	}
}

func (s *session) log(v ...interface{})                 { s.stor.Log(fmt.Sprint(v...)) }
func (s *session) logf(format string, v ...interface{}) { s.stor.Log(fmt.Sprintf(format, v...)) }

// File utils.

func (s *session) newTemp() storage.FileDesc {
	num := atomic.AddInt64(&s.stTempFileNum, 1) - 1
	return storage.FileDesc{Type: storage.TypeTemp, Num: num}
}

// Session state.

const (
	// maxCachedNumber represents the maximum number of version tasks
	// that can be cached in the ref loop.
	maxCachedNumber = 256

	// maxCachedTime represents the maximum time for ref loop to cache
	// a version task.
	maxCachedTime = 5 * time.Minute
)

// vDelta indicates the change information between the next version
// and the currently specified version
type vDelta struct {
	vid     int64
	added   []int64
	deleted []int64
}

// vTask defines a version task for either reference or release.
type vTask struct {
	vid     int64
	files   []tFiles
	created time.Time
}

func (s *session) refLoop() {
	var (
		fileRef    = make(map[int64]int)    // Table file reference counter
		ref        = make(map[int64]*vTask) // Current referencing version store
		deltas     = make(map[int64]*vDelta)
		referenced = make(map[int64]struct{})
		released   = make(map[int64]*vDelta)  // Released version that waiting for processing
		abandoned  = make(map[int64]struct{}) // Abandoned version id
		next, last int64
	)
	// addFileRef adds file reference counter with specified file number and
	// reference value
	addFileRef := func(fnum int64, ref int) int {
		ref += fileRef[fnum]
		if ref > 0 {
			fileRef[fnum] = ref
		} else if ref == 0 {
			delete(fileRef, fnum)
		} else {
			panic(fmt.Sprintf("negative ref: %v", fnum))
		}
		return ref
	}
	// skipAbandoned skips useless abandoned version id.
	skipAbandoned := func() bool {
		if _, exist := abandoned[next]; exist {
			delete(abandoned, next)
			return true
		}
		return false
	}
	// applyDelta applies version change to current file reference.
	applyDelta := func(d *vDelta) {
		for _, t := range d.added {
			addFileRef(t, 1)
		}
		for _, t := range d.deleted {
			if addFileRef(t, -1) == 0 {
				s.tops.remove(storage.FileDesc{Type: storage.TypeTable, Num: t})
			}
		}
	}

	timer := time.NewTimer(0)
	<-timer.C // discard the initial tick
	defer timer.Stop()

	// processTasks processes version tasks in strict order.
	//
	// If we want to use delta to reduce the cost of file references and dereferences,
	// we must strictly follow the id of the version, otherwise some files that are
	// being referenced will be deleted.
	//
	// In addition, some db operations (such as iterators) may cause a version to be
	// referenced for a long time. In order to prevent such operations from blocking
	// the entire processing queue, we will properly convert some of the version tasks
	// into full file references and releases.
	processTasks := func() {
		timer.Reset(maxCachedTime)
		// Make sure we don't cache too many version tasks.
		for {
			// Skip any abandoned version number to prevent blocking processing.
			if skipAbandoned() {
				next += 1
				continue
			}
			// Don't bother the version that has been released.
			if _, exist := released[next]; exist {
				break
			}
			// Ensure the specified version has been referenced.
			if _, exist := ref[next]; !exist {
				break
			}
			if last-next < maxCachedNumber && time.Since(ref[next].created) < maxCachedTime {
				break
			}
			// Convert version task into full file references and releases mode.
			// Reference version(i+1) first and wait version(i) to release.
			// FileRef(i+1) = FileRef(i) + Delta(i)
			for _, tt := range ref[next].files {
				for _, t := range tt {
					addFileRef(t.fd.Num, 1)
				}
			}
			// Note, if some compactions take a long time, even more than 5 minutes,
			// we may miss the corresponding delta information here.
			// Fortunately it will not affect the correctness of the file reference,
			// and we can apply the delta once we receive it.
			if d := deltas[next]; d != nil {
				applyDelta(d)
			}
			referenced[next] = struct{}{}
			delete(ref, next)
			delete(deltas, next)
			next += 1
		}

		// Use delta information to process all released versions.
		for {
			if skipAbandoned() {
				next += 1
				continue
			}
			if d, exist := released[next]; exist {
				if d != nil {
					applyDelta(d)
				}
				delete(released, next)
				next += 1
				continue
			}
			return
		}
	}

	for {
		processTasks()

		select {
		case t := <-s.refCh:
			if _, exist := ref[t.vid]; exist {
				panic("duplicate reference request")
			}
			ref[t.vid] = t
			if t.vid > last {
				last = t.vid
			}

		case d := <-s.deltaCh:
			if _, exist := ref[d.vid]; !exist {
				if _, exist2 := referenced[d.vid]; !exist2 {
					panic("invalid release request")
				}
				// The reference opt is already expired, apply
				// delta here.
				applyDelta(d)
				continue
			}
			deltas[d.vid] = d

		case t := <-s.relCh:
			if _, exist := referenced[t.vid]; exist {
				for _, tt := range t.files {
					for _, t := range tt {
						if addFileRef(t.fd.Num, -1) == 0 {
							s.tops.remove(t.fd)
						}
					}
				}
				delete(referenced, t.vid)
				continue
			}
			if _, exist := ref[t.vid]; !exist {
				panic("invalid release request")
			}
			released[t.vid] = deltas[t.vid]
			delete(deltas, t.vid)
			delete(ref, t.vid)

		case id := <-s.abandon:
			if id >= next {
				abandoned[id] = struct{}{}
			}

		case <-timer.C:

		case r := <-s.fileRefCh:
			ref := make(map[int64]int)
			for f, c := range fileRef {
				ref[f] = c
			}
			r <- ref

		case <-s.closeC:
			s.closeW.Done()
			return
		}
	}
}

// Get current version. This will incr version ref, must call
// version.release (exactly once) after use.
func (s *session) version() *version {
	s.vmu.Lock()
	defer s.vmu.Unlock()
	s.stVersion.incref()
	return s.stVersion
}

func (s *session) tLen(level int) int {
	s.vmu.Lock()
	defer s.vmu.Unlock()
	return s.stVersion.tLen(level)
}

// Set current version to v.
func (s *session) setVersion(r *sessionRecord, v *version) {
	s.vmu.Lock()
	defer s.vmu.Unlock()
	// Hold by session. It is important to call this first before releasing
	// current version, otherwise the still used files might get released.
	v.incref()
	if s.stVersion != nil {
		if r != nil {
			var (
				added   = make([]int64, 0, len(r.addedTables))
				deleted = make([]int64, 0, len(r.deletedTables))
			)
			for _, t := range r.addedTables {
				added = append(added, t.num)
			}
			for _, t := range r.deletedTables {
				deleted = append(deleted, t.num)
			}
			select {
			case s.deltaCh <- &vDelta{vid: s.stVersion.id, added: added, deleted: deleted}:
			case <-v.s.closeC:
				s.log("reference loop already exist")
			}
		}
		// Release current version.
		s.stVersion.releaseNB()
	}
	s.stVersion = v
}

// Get current unused file number.
func (s *session) nextFileNum() int64 {
	return atomic.LoadInt64(&s.stNextFileNum)
}

// Set current unused file number to num.
func (s *session) setNextFileNum(num int64) {
	atomic.StoreInt64(&s.stNextFileNum, num)
}

// Mark file number as used.
func (s *session) markFileNum(num int64) {
	nextFileNum := num + 1
	for {
		old, x := atomic.LoadInt64(&s.stNextFileNum), nextFileNum
		if old > x {
			x = old
		}
		if atomic.CompareAndSwapInt64(&s.stNextFileNum, old, x) {
			break
		}
	}
}

// Allocate a file number.
func (s *session) allocFileNum() int64 {
	return atomic.AddInt64(&s.stNextFileNum, 1) - 1
}

// Reuse given file number.
func (s *session) reuseFileNum(num int64) {
	for {
		old, x := atomic.LoadInt64(&s.stNextFileNum), num
		if old != x+1 {
			x = old
		}
		if atomic.CompareAndSwapInt64(&s.stNextFileNum, old, x) {
			break
		}
	}
}

// Set compaction ptr at given level; need external synchronization.
func (s *session) setCompPtr(level int, ik internalKey) {
	if level >= len(s.stCompPtrs) {
		newCompPtrs := make([]internalKey, level+1)
		copy(newCompPtrs, s.stCompPtrs)
		s.stCompPtrs = newCompPtrs
	}
	s.stCompPtrs[level] = append(internalKey{}, ik...)
}

// Get compaction ptr at given level; need external synchronization.
func (s *session) getCompPtr(level int) internalKey {
	if level >= len(s.stCompPtrs) {
		return nil
	}
	return s.stCompPtrs[level]
}

// Manifest related utils.

// Fill given session record obj with current states; need external
// synchronization.
func (s *session) fillRecord(r *sessionRecord, snapshot bool) {
	r.setNextFileNum(s.nextFileNum())

	if snapshot {
		if !r.has(recJournalNum) {
			r.setJournalNum(s.stJournalNum)
		}

		if !r.has(recSeqNum) {
			r.setSeqNum(s.stSeqNum)
		}

		for level, ik := range s.stCompPtrs {
			if ik != nil {
				r.addCompPtr(level, ik)
			}
		}

		r.setComparer(s.icmp.uName())
	}
}

// Mark if record has been committed, this will update session state;
// need external synchronization.
func (s *session) recordCommited(rec *sessionRecord) {
	if rec.has(recJournalNum) {
		s.stJournalNum = rec.journalNum
	}

	if rec.has(recPrevJournalNum) {
		s.stPrevJournalNum = rec.prevJournalNum
	}

	if rec.has(recSeqNum) {
		s.stSeqNum = rec.seqNum
	}

	for _, r := range rec.compPtrs {
		s.setCompPtr(r.level, internalKey(r.ikey))
	}
}

// Create a new manifest file; need external synchronization.
func (s *session) newManifest(rec *sessionRecord, v *version) (err error) {
	fd := storage.FileDesc{Type: storage.TypeManifest, Num: s.allocFileNum()}
	writer, err := s.stor.Create(fd)
	if err != nil {
		return
	}
	jw := journal.NewWriter(writer)

	if v == nil {
		v = s.version()
		defer v.release()
	}
	if rec == nil {
		rec = &sessionRecord{}
	}
	s.fillRecord(rec, true)
	v.fillRecord(rec)

	defer func() {
		if err == nil {
			s.recordCommited(rec)
			if s.manifest != nil {
				s.manifest.Close()
			}
			if s.manifestWriter != nil {
				s.manifestWriter.Close()
			}
			if !s.manifestFd.Zero() {
				s.stor.Remove(s.manifestFd)
			}
			s.manifestFd = fd
			s.manifestWriter = writer
			s.manifest = jw
		} else {
			writer.Close()
			s.stor.Remove(fd)
			s.reuseFileNum(fd.Num)
		}
	}()

	w, err := jw.Next()
	if err != nil {
		return
	}
	err = rec.encode(w)
	if err != nil {
		return
	}
	err = jw.Flush()
	if err != nil {
		return
	}
	if !s.o.GetNoSync() {
		err = writer.Sync()
		if err != nil {
			return
		}
	}
	err = s.stor.SetMeta(fd)
	return
}

// Flush record to disk.
func (s *session) flushManifest(rec *sessionRecord) (err error) {
	s.fillRecord(rec, false)
	w, err := s.manifest.Next()
	if err != nil {
		return
	}
	err = rec.encode(w)
	if err != nil {
		return
	}
	err = s.manifest.Flush()
	if err != nil {
		return
	}
	if !s.o.GetNoSync() {
		err = s.manifestWriter.Sync()
		if err != nil {
			return
		}
	}
	s.recordCommited(rec)
	return
}

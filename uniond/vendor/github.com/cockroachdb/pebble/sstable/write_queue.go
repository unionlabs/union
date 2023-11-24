package sstable

import (
	"sync"

	"github.com/cockroachdb/pebble/internal/base"
)

type writeTask struct {
	// Since writeTasks are pooled, the compressionDone channel will be re-used.
	// It is necessary that any writes to the channel have already been read,
	// before adding the writeTask back to the pool.
	compressionDone chan bool
	buf             *dataBlockBuf
	// If this is not nil, then this index block will be flushed.
	flushableIndexBlock *indexBlockBuf
	// currIndexBlock is the index block on which indexBlock.add must be called.
	currIndexBlock *indexBlockBuf
	indexEntrySep  InternalKey
	// inflightIndexEntrySize is used to decrement Writer.indexBlock.sizeEstimate.inflightSize.
	indexInflightSize int
	// If the index block is finished, then we set the finishedIndexProps here.
	finishedIndexProps []byte
}

// It is not the responsibility of the writeTask to clear the
// task.flushableIndexBlock, and task.buf.
func (task *writeTask) clear() {
	*task = writeTask{
		indexEntrySep:   base.InvalidInternalKey,
		compressionDone: task.compressionDone,
	}
}

// Note that only the Writer client goroutine will be adding tasks to the writeQueue.
// Both the Writer client and the compression goroutines will be able to write to
// writeTask.compressionDone to indicate that the compression job associated with
// a writeTask has finished.
type writeQueue struct {
	tasks  chan *writeTask
	wg     sync.WaitGroup
	writer *Writer

	// err represents an error which is encountered when the write queue attempts
	// to write a block to disk. The error is stored here to skip unnecessary block
	// writes once the first error is encountered.
	err    error
	closed bool
}

func newWriteQueue(size int, writer *Writer) *writeQueue {
	w := &writeQueue{}
	w.tasks = make(chan *writeTask, size)
	w.writer = writer

	w.wg.Add(1)
	go w.runWorker()
	return w
}

func (w *writeQueue) performWrite(task *writeTask) error {
	var bh BlockHandle
	var bhp BlockHandleWithProperties

	var err error
	if bh, err = w.writer.writeCompressedBlock(task.buf.compressed, task.buf.tmp[:]); err != nil {
		return err
	}

	bhp = BlockHandleWithProperties{BlockHandle: bh, Props: task.buf.dataBlockProps}
	if err = w.writer.addIndexEntry(
		task.indexEntrySep, bhp, task.buf.tmp[:], task.flushableIndexBlock, task.currIndexBlock,
		task.indexInflightSize, task.finishedIndexProps); err != nil {
		return err
	}

	return nil
}

// It is necessary to ensure that none of the buffers in the writeTask,
// dataBlockBuf, indexBlockBuf, are pointed to by another struct.
func (w *writeQueue) releaseBuffers(task *writeTask) {
	task.buf.clear()
	dataBlockBufPool.Put(task.buf)

	// This index block is no longer used by the Writer, so we can add it back
	// to the pool.
	if task.flushableIndexBlock != nil {
		task.flushableIndexBlock.clear()
		indexBlockBufPool.Put(task.flushableIndexBlock)
	}

	task.clear()
	writeTaskPool.Put(task)
}

func (w *writeQueue) runWorker() {
	for task := range w.tasks {
		<-task.compressionDone

		if w.err == nil {
			w.err = w.performWrite(task)
		}

		w.releaseBuffers(task)
	}
	w.wg.Done()
}

func (w *writeQueue) add(task *writeTask) {
	w.tasks <- task
}

// addSync will perform the writeTask synchronously with the caller goroutine. Calls to addSync
// are no longer valid once writeQueue.add has been called at least once.
func (w *writeQueue) addSync(task *writeTask) error {
	// This should instantly return without blocking.
	<-task.compressionDone

	if w.err == nil {
		w.err = w.performWrite(task)
	}

	w.releaseBuffers(task)

	return w.err
}

// finish should only be called once no more tasks will be added to the writeQueue.
// finish will return any error which was encountered while tasks were processed.
func (w *writeQueue) finish() error {
	if w.closed {
		return w.err
	}

	close(w.tasks)
	w.wg.Wait()
	w.closed = true
	return w.err
}

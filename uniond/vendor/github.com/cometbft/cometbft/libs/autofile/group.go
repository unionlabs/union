package autofile

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/cometbft/cometbft/libs/service"
)

const (
	defaultGroupCheckDuration = 5000 * time.Millisecond
	defaultHeadSizeLimit      = 10 * 1024 * 1024       // 10MB
	defaultTotalSizeLimit     = 1 * 1024 * 1024 * 1024 // 1GB
	maxFilesToRemove          = 4                      // needs to be greater than 1
)

/*
You can open a Group to keep restrictions on an AutoFile, like
the maximum size of each chunk, and/or the total amount of bytes
stored in the group.

The first file to be written in the Group.Dir is the head file.

	Dir/
	- <HeadPath>

Once the Head file reaches the size limit, it will be rotated.

	Dir/
	- <HeadPath>.000   // First rolled file
	- <HeadPath>       // New head path, starts empty.
										 // The implicit index is 001.

As more files are written, the index numbers grow...

	Dir/
	- <HeadPath>.000   // First rolled file
	- <HeadPath>.001   // Second rolled file
	- ...
	- <HeadPath>       // New head path

The Group can also be used to binary-search for some line,
assuming that marker lines are written occasionally.
*/
type Group struct {
	service.BaseService

	ID                 string
	Head               *AutoFile // The head AutoFile to write to
	headBuf            *bufio.Writer
	Dir                string // Directory that contains .Head
	ticker             *time.Ticker
	mtx                sync.Mutex
	headSizeLimit      int64
	totalSizeLimit     int64
	groupCheckDuration time.Duration
	minIndex           int // Includes head
	maxIndex           int // Includes head, where Head will move to

	// close this when the processTicks routine is done.
	// this ensures we can cleanup the dir after calling Stop
	// and the routine won't be trying to access it anymore
	doneProcessTicks chan struct{}

	// TODO: When we start deleting files, we need to start tracking GroupReaders
	// and their dependencies.
}

// OpenGroup creates a new Group with head at headPath. It returns an error if
// it fails to open head file.
func OpenGroup(headPath string, groupOptions ...func(*Group)) (*Group, error) {
	dir, err := filepath.Abs(filepath.Dir(headPath))
	if err != nil {
		return nil, err
	}
	head, err := OpenAutoFile(headPath)
	if err != nil {
		return nil, err
	}

	g := &Group{
		ID:                 "group:" + head.ID,
		Head:               head,
		headBuf:            bufio.NewWriterSize(head, 4096*10),
		Dir:                dir,
		headSizeLimit:      defaultHeadSizeLimit,
		totalSizeLimit:     defaultTotalSizeLimit,
		groupCheckDuration: defaultGroupCheckDuration,
		minIndex:           0,
		maxIndex:           0,
		doneProcessTicks:   make(chan struct{}),
	}

	for _, option := range groupOptions {
		option(g)
	}

	g.BaseService = *service.NewBaseService(nil, "Group", g)

	gInfo := g.readGroupInfo()
	g.minIndex = gInfo.MinIndex
	g.maxIndex = gInfo.MaxIndex
	return g, nil
}

// GroupCheckDuration allows you to overwrite default groupCheckDuration.
func GroupCheckDuration(duration time.Duration) func(*Group) {
	return func(g *Group) {
		g.groupCheckDuration = duration
	}
}

// GroupHeadSizeLimit allows you to overwrite default head size limit - 10MB.
func GroupHeadSizeLimit(limit int64) func(*Group) {
	return func(g *Group) {
		g.headSizeLimit = limit
	}
}

// GroupTotalSizeLimit allows you to overwrite default total size limit of the group - 1GB.
func GroupTotalSizeLimit(limit int64) func(*Group) {
	return func(g *Group) {
		g.totalSizeLimit = limit
	}
}

// OnStart implements service.Service by starting the goroutine that checks file
// and group limits.
func (g *Group) OnStart() error {
	g.ticker = time.NewTicker(g.groupCheckDuration)
	go g.processTicks()
	return nil
}

// OnStop implements service.Service by stopping the goroutine described above.
// NOTE: g.Head must be closed separately using Close.
func (g *Group) OnStop() {
	g.ticker.Stop()
	if err := g.FlushAndSync(); err != nil {
		g.Logger.Error("Error flushin to disk", "err", err)
	}
}

// Wait blocks until all internal goroutines are finished. Supposed to be
// called after Stop.
func (g *Group) Wait() {
	// wait for processTicks routine to finish
	<-g.doneProcessTicks
}

// Close closes the head file. The group must be stopped by this moment.
func (g *Group) Close() {
	if err := g.FlushAndSync(); err != nil {
		g.Logger.Error("Error flushin to disk", "err", err)
	}

	g.mtx.Lock()
	_ = g.Head.closeFile()
	g.mtx.Unlock()
}

// HeadSizeLimit returns the current head size limit.
func (g *Group) HeadSizeLimit() int64 {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	return g.headSizeLimit
}

// TotalSizeLimit returns total size limit of the group.
func (g *Group) TotalSizeLimit() int64 {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	return g.totalSizeLimit
}

// MaxIndex returns index of the last file in the group.
func (g *Group) MaxIndex() int {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	return g.maxIndex
}

// MinIndex returns index of the first file in the group.
func (g *Group) MinIndex() int {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	return g.minIndex
}

// Write writes the contents of p into the current head of the group. It
// returns the number of bytes written. If nn < len(p), it also returns an
// error explaining why the write is short.
// NOTE: Writes are buffered so they don't write synchronously
// TODO: Make it halt if space is unavailable
func (g *Group) Write(p []byte) (nn int, err error) {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	return g.headBuf.Write(p)
}

// WriteLine writes line into the current head of the group. It also appends "\n".
// NOTE: Writes are buffered so they don't write synchronously
// TODO: Make it halt if space is unavailable
func (g *Group) WriteLine(line string) error {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	_, err := g.headBuf.Write([]byte(line + "\n"))
	return err
}

// Buffered returns the size of the currently buffered data.
func (g *Group) Buffered() int {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	return g.headBuf.Buffered()
}

// FlushAndSync writes any buffered data to the underlying file and commits the
// current content of the file to stable storage (fsync).
func (g *Group) FlushAndSync() error {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	err := g.headBuf.Flush()
	if err == nil {
		err = g.Head.Sync()
	}
	return err
}

func (g *Group) processTicks() {
	defer close(g.doneProcessTicks)
	for {
		select {
		case <-g.ticker.C:
			g.checkHeadSizeLimit()
			g.checkTotalSizeLimit()
		case <-g.Quit():
			return
		}
	}
}

// NOTE: this function is called manually in tests.
func (g *Group) checkHeadSizeLimit() {
	limit := g.HeadSizeLimit()
	if limit == 0 {
		return
	}
	size, err := g.Head.Size()
	if err != nil {
		g.Logger.Error("Group's head may grow without bound", "head", g.Head.Path, "err", err)
		return
	}
	if size >= limit {
		g.RotateFile()
	}
}

func (g *Group) checkTotalSizeLimit() {
	limit := g.TotalSizeLimit()
	if limit == 0 {
		return
	}

	gInfo := g.readGroupInfo()
	totalSize := gInfo.TotalSize
	for i := 0; i < maxFilesToRemove; i++ {
		index := gInfo.MinIndex + i
		if totalSize < limit {
			return
		}
		if index == gInfo.MaxIndex {
			// Special degenerate case, just do nothing.
			g.Logger.Error("Group's head may grow without bound", "head", g.Head.Path)
			return
		}
		pathToRemove := filePathForIndex(g.Head.Path, index, gInfo.MaxIndex)
		fInfo, err := os.Stat(pathToRemove)
		if err != nil {
			g.Logger.Error("Failed to fetch info for file", "file", pathToRemove)
			continue
		}
		err = os.Remove(pathToRemove)
		if err != nil {
			g.Logger.Error("Failed to remove path", "path", pathToRemove)
			return
		}
		totalSize -= fInfo.Size()
	}
}

// RotateFile causes group to close the current head and assign it some index.
// Note it does not create a new head.
func (g *Group) RotateFile() {
	g.mtx.Lock()
	defer g.mtx.Unlock()

	headPath := g.Head.Path

	if err := g.headBuf.Flush(); err != nil {
		panic(err)
	}

	if err := g.Head.Sync(); err != nil {
		panic(err)
	}

	if err := g.Head.closeFile(); err != nil {
		panic(err)
	}

	indexPath := filePathForIndex(headPath, g.maxIndex, g.maxIndex+1)
	if err := os.Rename(headPath, indexPath); err != nil {
		panic(err)
	}

	g.maxIndex++
}

// NewReader returns a new group reader.
// CONTRACT: Caller must close the returned GroupReader.
func (g *Group) NewReader(index int) (*GroupReader, error) {
	r := newGroupReader(g)
	err := r.SetIndex(index)
	if err != nil {
		return nil, err
	}
	return r, nil
}

// GroupInfo holds information about the group.
type GroupInfo struct {
	MinIndex  int   // index of the first file in the group, including head
	MaxIndex  int   // index of the last file in the group, including head
	TotalSize int64 // total size of the group
	HeadSize  int64 // size of the head
}

// Returns info after scanning all files in g.Head's dir.
func (g *Group) ReadGroupInfo() GroupInfo {
	g.mtx.Lock()
	defer g.mtx.Unlock()
	return g.readGroupInfo()
}

// Index includes the head.
// CONTRACT: caller should have called g.mtx.Lock
func (g *Group) readGroupInfo() GroupInfo {
	groupDir := filepath.Dir(g.Head.Path)
	headBase := filepath.Base(g.Head.Path)
	var minIndex, maxIndex int = -1, -1
	var totalSize, headSize int64 = 0, 0

	dir, err := os.Open(groupDir)
	if err != nil {
		panic(err)
	}
	defer dir.Close()
	fiz, err := dir.Readdir(0)
	if err != nil {
		panic(err)
	}

	// For each file in the directory, filter by pattern
	for _, fileInfo := range fiz {
		if fileInfo.Name() == headBase {
			fileSize := fileInfo.Size()
			totalSize += fileSize
			headSize = fileSize
			continue
		} else if strings.HasPrefix(fileInfo.Name(), headBase) {
			fileSize := fileInfo.Size()
			totalSize += fileSize
			indexedFilePattern := regexp.MustCompile(`^.+\.([0-9]{3,})$`)
			submatch := indexedFilePattern.FindSubmatch([]byte(fileInfo.Name()))
			if len(submatch) != 0 {
				// Matches
				fileIndex, err := strconv.Atoi(string(submatch[1]))
				if err != nil {
					panic(err)
				}
				if maxIndex < fileIndex {
					maxIndex = fileIndex
				}
				if minIndex == -1 || fileIndex < minIndex {
					minIndex = fileIndex
				}
			}
		}
	}

	// Now account for the head.
	if minIndex == -1 {
		// If there were no numbered files,
		// then the head is index 0.
		minIndex, maxIndex = 0, 0
	} else {
		// Otherwise, the head file is 1 greater
		maxIndex++
	}
	return GroupInfo{minIndex, maxIndex, totalSize, headSize}
}

func filePathForIndex(headPath string, index int, maxIndex int) string {
	if index == maxIndex {
		return headPath
	}
	return fmt.Sprintf("%v.%03d", headPath, index)
}

//--------------------------------------------------------------------------------

// GroupReader provides an interface for reading from a Group.
type GroupReader struct {
	*Group
	mtx       sync.Mutex
	curIndex  int
	curFile   *os.File
	curReader *bufio.Reader
	curLine   []byte
}

func newGroupReader(g *Group) *GroupReader {
	return &GroupReader{
		Group:     g,
		curIndex:  0,
		curFile:   nil,
		curReader: nil,
		curLine:   nil,
	}
}

// Close closes the GroupReader by closing the cursor file.
func (gr *GroupReader) Close() error {
	gr.mtx.Lock()
	defer gr.mtx.Unlock()

	if gr.curReader != nil {
		err := gr.curFile.Close()
		gr.curIndex = 0
		gr.curReader = nil
		gr.curFile = nil
		gr.curLine = nil
		return err
	}
	return nil
}

// Read implements io.Reader, reading bytes from the current Reader
// incrementing index until enough bytes are read.
func (gr *GroupReader) Read(p []byte) (n int, err error) {
	lenP := len(p)
	if lenP == 0 {
		return 0, errors.New("given empty slice")
	}

	gr.mtx.Lock()
	defer gr.mtx.Unlock()

	// Open file if not open yet
	if gr.curReader == nil {
		if err = gr.openFile(gr.curIndex); err != nil {
			return 0, err
		}
	}

	// Iterate over files until enough bytes are read
	var nn int
	for {
		nn, err = gr.curReader.Read(p[n:])
		n += nn
		switch {
		case err == io.EOF:
			if n >= lenP {
				return n, nil
			}
			// Open the next file
			if err1 := gr.openFile(gr.curIndex + 1); err1 != nil {
				return n, err1
			}
		case err != nil:
			return n, err
		case nn == 0: // empty file
			return n, err
		}
	}
}

// IF index > gr.Group.maxIndex, returns io.EOF
// CONTRACT: caller should hold gr.mtx
func (gr *GroupReader) openFile(index int) error {
	// Lock on Group to ensure that head doesn't move in the meanwhile.
	gr.Group.mtx.Lock()
	defer gr.Group.mtx.Unlock()

	if index > gr.Group.maxIndex {
		return io.EOF
	}

	curFilePath := filePathForIndex(gr.Head.Path, index, gr.Group.maxIndex)
	curFile, err := os.OpenFile(curFilePath, os.O_RDONLY|os.O_CREATE, autoFilePerms)
	if err != nil {
		return err
	}
	curReader := bufio.NewReader(curFile)

	// Update gr.cur*
	if gr.curFile != nil {
		gr.curFile.Close() // TODO return error?
	}
	gr.curIndex = index
	gr.curFile = curFile
	gr.curReader = curReader
	gr.curLine = nil
	return nil
}

// CurIndex returns cursor's file index.
func (gr *GroupReader) CurIndex() int {
	gr.mtx.Lock()
	defer gr.mtx.Unlock()
	return gr.curIndex
}

// SetIndex sets the cursor's file index to index by opening a file at this
// position.
func (gr *GroupReader) SetIndex(index int) error {
	gr.mtx.Lock()
	defer gr.mtx.Unlock()
	return gr.openFile(index)
}

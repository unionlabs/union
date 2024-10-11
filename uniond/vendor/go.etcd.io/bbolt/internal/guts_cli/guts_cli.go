package guts_cli

// Low level access to pages / data-structures of the bbolt file.

import (
	"errors"
	"fmt"
	"io"
	"os"

	"go.etcd.io/bbolt/internal/common"
)

var (
	// ErrCorrupt is returned when a checking a data file finds errors.
	ErrCorrupt = errors.New("invalid value")
)

// ReadPage reads Page info & full Page data from a path.
// This is not transactionally safe.
func ReadPage(path string, pageID uint64) (*common.Page, []byte, error) {
	// Find Page size.
	pageSize, hwm, err := ReadPageAndHWMSize(path)
	if err != nil {
		return nil, nil, fmt.Errorf("read Page size: %s", err)
	}

	// Open database file.
	f, err := os.Open(path)
	if err != nil {
		return nil, nil, err
	}
	defer f.Close()

	// Read one block into buffer.
	buf := make([]byte, pageSize)
	if n, err := f.ReadAt(buf, int64(pageID*pageSize)); err != nil {
		return nil, nil, err
	} else if n != len(buf) {
		return nil, nil, io.ErrUnexpectedEOF
	}

	// Determine total number of blocks.
	p := common.LoadPage(buf)
	if p.Id() != common.Pgid(pageID) {
		return nil, nil, fmt.Errorf("error: %w due to unexpected Page id: %d != %d", ErrCorrupt, p.Id(), pageID)
	}
	overflowN := p.Overflow()
	if overflowN >= uint32(hwm)-3 { // we exclude 2 Meta pages and the current Page.
		return nil, nil, fmt.Errorf("error: %w, Page claims to have %d overflow pages (>=hwm=%d). Interrupting to avoid risky OOM", ErrCorrupt, overflowN, hwm)
	}

	if overflowN == 0 {
		return p, buf, nil
	}

	// Re-read entire Page (with overflow) into buffer.
	buf = make([]byte, (uint64(overflowN)+1)*pageSize)
	if n, err := f.ReadAt(buf, int64(pageID*pageSize)); err != nil {
		return nil, nil, err
	} else if n != len(buf) {
		return nil, nil, io.ErrUnexpectedEOF
	}
	p = common.LoadPage(buf)
	if p.Id() != common.Pgid(pageID) {
		return nil, nil, fmt.Errorf("error: %w due to unexpected Page id: %d != %d", ErrCorrupt, p.Id(), pageID)
	}

	return p, buf, nil
}

func WritePage(path string, pageBuf []byte) error {
	page := common.LoadPage(pageBuf)
	pageSize, _, err := ReadPageAndHWMSize(path)
	if err != nil {
		return err
	}
	expectedLen := pageSize * (uint64(page.Overflow()) + 1)
	if expectedLen != uint64(len(pageBuf)) {
		return fmt.Errorf("WritePage: len(buf):%d != pageSize*(overflow+1):%d", len(pageBuf), expectedLen)
	}
	f, err := os.OpenFile(path, os.O_WRONLY, 0)
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = f.WriteAt(pageBuf, int64(page.Id())*int64(pageSize))
	return err
}

// ReadPageAndHWMSize reads Page size and HWM (id of the last+1 Page).
// This is not transactionally safe.
func ReadPageAndHWMSize(path string) (uint64, common.Pgid, error) {
	// Open database file.
	f, err := os.Open(path)
	if err != nil {
		return 0, 0, err
	}
	defer f.Close()

	// Read 4KB chunk.
	buf := make([]byte, 4096)
	if _, err := io.ReadFull(f, buf); err != nil {
		return 0, 0, err
	}

	// Read Page size from metadata.
	m := common.LoadPageMeta(buf)
	if m.Magic() != common.Magic {
		return 0, 0, fmt.Errorf("the Meta Page has wrong (unexpected) magic")
	}
	return uint64(m.PageSize()), common.Pgid(m.Pgid()), nil
}

// GetRootPage returns the root-page (according to the most recent transaction).
func GetRootPage(path string) (root common.Pgid, activeMeta common.Pgid, err error) {
	_, buf0, err0 := ReadPage(path, 0)
	if err0 != nil {
		return 0, 0, err0
	}
	m0 := common.LoadPageMeta(buf0)
	_, buf1, err1 := ReadPage(path, 1)
	if err1 != nil {
		return 0, 1, err1
	}
	m1 := common.LoadPageMeta(buf1)
	if m0.Txid() < m1.Txid() {
		return m1.RootBucket().RootPage(), 1, nil
	} else {
		return m0.RootBucket().RootPage(), 0, nil
	}
}

package surgeon

import (
	"fmt"

	"go.etcd.io/bbolt/internal/common"
	"go.etcd.io/bbolt/internal/guts_cli"
)

func CopyPage(path string, srcPage common.Pgid, target common.Pgid) error {
	p1, d1, err1 := guts_cli.ReadPage(path, uint64(srcPage))
	if err1 != nil {
		return err1
	}
	p1.SetId(target)
	return guts_cli.WritePage(path, d1)
}

func ClearPage(path string, pgId common.Pgid) (bool, error) {
	return ClearPageElements(path, pgId, 0, -1, false)
}

// ClearPageElements supports clearing elements in both branch and leaf
// pages. Note if the ${abandonFreelist} is true, the freelist may be cleaned
// in the meta pages in the following two cases, and bbolt needs to scan the
// db to reconstruct free list. It may cause some delay on next startup,
// depending on the db size.
//  1. Any branch elements are cleared;
//  2. An object saved in overflow pages is cleared;
//
// Usually ${abandonFreelist} defaults to false, it means it will not clear the
// freelist in meta pages automatically. Users will receive a warning message
// to remind them to explicitly execute `bbolt surgery abandom-freelist`
// afterwards; the first return parameter will be true in such case. But if
// the freelist isn't synced at all, no warning message will be displayed.
func ClearPageElements(path string, pgId common.Pgid, start, end int, abandonFreelist bool) (bool, error) {
	// Read the page
	p, buf, err := guts_cli.ReadPage(path, uint64(pgId))
	if err != nil {
		return false, fmt.Errorf("ReadPage failed: %w", err)
	}

	if !p.IsLeafPage() && !p.IsBranchPage() {
		return false, fmt.Errorf("can't clear elements in %q page", p.Typ())
	}

	elementCnt := int(p.Count())

	if elementCnt == 0 {
		return false, nil
	}

	if start < 0 || start >= elementCnt {
		return false, fmt.Errorf("the start index (%d) is out of range [0, %d)", start, elementCnt)
	}

	if (end < 0 || end > elementCnt) && end != -1 {
		return false, fmt.Errorf("the end index (%d) is out of range [0, %d]", end, elementCnt)
	}

	if start > end && end != -1 {
		return false, fmt.Errorf("the start index (%d) is bigger than the end index (%d)", start, end)
	}

	if start == end {
		return false, fmt.Errorf("invalid: the start index (%d) is equal to the end index (%d)", start, end)
	}

	preOverflow := p.Overflow()

	var (
		dataWritten uint32
	)
	if end == int(p.Count()) || end == -1 {
		inodes := common.ReadInodeFromPage(p)
		inodes = inodes[:start]

		p.SetCount(uint16(start))
		// no need to write inode & data again, we just need to get
		// the data size which will be kept.
		dataWritten = common.UsedSpaceInPage(inodes, p)
	} else {
		inodes := common.ReadInodeFromPage(p)
		inodes = append(inodes[:start], inodes[end:]...)

		p.SetCount(uint16(len(inodes)))
		dataWritten = common.WriteInodeToPage(inodes, p)
	}

	pageSize, _, err := guts_cli.ReadPageAndHWMSize(path)
	if err != nil {
		return false, fmt.Errorf("ReadPageAndHWMSize failed: %w", err)
	}
	if dataWritten%uint32(pageSize) == 0 {
		p.SetOverflow(dataWritten/uint32(pageSize) - 1)
	} else {
		p.SetOverflow(dataWritten / uint32(pageSize))
	}

	datasz := pageSize * (uint64(p.Overflow()) + 1)
	if err := guts_cli.WritePage(path, buf[0:datasz]); err != nil {
		return false, fmt.Errorf("WritePage failed: %w", err)
	}

	if preOverflow != p.Overflow() || p.IsBranchPage() {
		if abandonFreelist {
			return false, ClearFreelist(path)
		}
		return true, nil
	}

	return false, nil
}

func ClearFreelist(path string) error {
	if err := clearFreelistInMetaPage(path, 0); err != nil {
		return fmt.Errorf("clearFreelist on meta page 0 failed: %w", err)
	}
	if err := clearFreelistInMetaPage(path, 1); err != nil {
		return fmt.Errorf("clearFreelist on meta page 1 failed: %w", err)
	}
	return nil
}

func clearFreelistInMetaPage(path string, pageId uint64) error {
	_, buf, err := guts_cli.ReadPage(path, pageId)
	if err != nil {
		return fmt.Errorf("ReadPage %d failed: %w", pageId, err)
	}

	meta := common.LoadPageMeta(buf)
	meta.SetFreelist(common.PgidNoFreelist)
	meta.SetChecksum(meta.Sum64())

	if err := guts_cli.WritePage(path, buf); err != nil {
		return fmt.Errorf("WritePage %d failed: %w", pageId, err)
	}

	return nil
}

// RevertMetaPage replaces the newer metadata page with the older.
// It usually means that one transaction is being lost. But frequently
// data corruption happens on the last transaction pages and the
// previous state is consistent.
func RevertMetaPage(path string) error {
	_, activeMetaPage, err := guts_cli.GetRootPage(path)
	if err != nil {
		return err
	}
	if activeMetaPage == 0 {
		return CopyPage(path, 1, 0)
	} else {
		return CopyPage(path, 0, 1)
	}
}

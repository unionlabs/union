package surgeon

// Library contains raw access to bbolt files for sake of testing or fixing of corrupted files.
//
// The library must not be used bbolt btree - just by CLI or tests.
// It's not optimized for performance.

import (
	"bytes"
	"fmt"

	"go.etcd.io/bbolt/internal/common"
	"go.etcd.io/bbolt/internal/guts_cli"
)

type XRay struct {
	path string
}

func NewXRay(path string) XRay {
	return XRay{path}
}

func (n XRay) traverse(stack []common.Pgid, callback func(page *common.Page, stack []common.Pgid) error) error {
	p, data, err := guts_cli.ReadPage(n.path, uint64(stack[len(stack)-1]))
	if err != nil {
		return fmt.Errorf("failed reading page (stack %v): %w", stack, err)
	}
	err = callback(p, stack)
	if err != nil {
		return fmt.Errorf("failed callback for page (stack %v): %w", stack, err)
	}
	switch p.Typ() {
	case "meta":
		{
			m := common.LoadPageMeta(data)
			r := m.RootBucket().RootPage()
			return n.traverse(append(stack, r), callback)
		}
	case "branch":
		{
			for i := uint16(0); i < p.Count(); i++ {
				bpe := p.BranchPageElement(i)
				if err := n.traverse(append(stack, bpe.Pgid()), callback); err != nil {
					return err
				}
			}
		}
	case "leaf":
		for i := uint16(0); i < p.Count(); i++ {
			lpe := p.LeafPageElement(i)
			if lpe.IsBucketEntry() {
				pgid := lpe.Bucket().RootPage()
				if pgid > 0 {
					if err := n.traverse(append(stack, pgid), callback); err != nil {
						return err
					}
				} else {
					inlinePage := lpe.Bucket().InlinePage(lpe.Value())
					if err := callback(inlinePage, stack); err != nil {
						return fmt.Errorf("failed callback for inline page  (stack %v): %w", stack, err)
					}
				}
			}
		}
	case "freelist":
		return nil
		// Free does not have children.
	}
	return nil
}

// FindPathsToKey finds all paths from root to the page that contains the given key.
// As it traverses multiple buckets, so in theory there might be multiple keys with the given name.
// Note: For simplicity it's currently implemented as traversing of the whole reachable tree.
// If key is a bucket name, a page-path referencing the key will be returned as well.
func (n XRay) FindPathsToKey(key []byte) ([][]common.Pgid, error) {
	var found [][]common.Pgid

	rootPage, _, err := guts_cli.GetRootPage(n.path)
	if err != nil {
		return nil, err
	}
	err = n.traverse([]common.Pgid{rootPage},
		func(page *common.Page, stack []common.Pgid) error {
			if page.Typ() == "leaf" {
				for i := uint16(0); i < page.Count(); i++ {
					if bytes.Equal(page.LeafPageElement(i).Key(), key) {
						var copyPath []common.Pgid
						copyPath = append(copyPath, stack...)
						found = append(found, copyPath)
					}
				}
			}
			return nil
		})
	if err != nil {
		return nil, err
	} else {
		return found, nil
	}
}

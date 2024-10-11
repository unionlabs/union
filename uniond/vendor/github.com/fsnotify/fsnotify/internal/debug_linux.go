package internal

import (
	"fmt"
	"os"
	"strings"
	"time"

	"golang.org/x/sys/unix"
)

func Debug(name string, mask uint32) {
	names := []struct {
		n string
		m uint32
	}{
		{"IN_ACCESS", unix.IN_ACCESS},
		{"IN_ALL_EVENTS", unix.IN_ALL_EVENTS},
		{"IN_ATTRIB", unix.IN_ATTRIB},
		{"IN_CLASSA_HOST", unix.IN_CLASSA_HOST},
		{"IN_CLASSA_MAX", unix.IN_CLASSA_MAX},
		{"IN_CLASSA_NET", unix.IN_CLASSA_NET},
		{"IN_CLASSA_NSHIFT", unix.IN_CLASSA_NSHIFT},
		{"IN_CLASSB_HOST", unix.IN_CLASSB_HOST},
		{"IN_CLASSB_MAX", unix.IN_CLASSB_MAX},
		{"IN_CLASSB_NET", unix.IN_CLASSB_NET},
		{"IN_CLASSB_NSHIFT", unix.IN_CLASSB_NSHIFT},
		{"IN_CLASSC_HOST", unix.IN_CLASSC_HOST},
		{"IN_CLASSC_NET", unix.IN_CLASSC_NET},
		{"IN_CLASSC_NSHIFT", unix.IN_CLASSC_NSHIFT},
		{"IN_CLOSE", unix.IN_CLOSE},
		{"IN_CLOSE_NOWRITE", unix.IN_CLOSE_NOWRITE},
		{"IN_CLOSE_WRITE", unix.IN_CLOSE_WRITE},
		{"IN_CREATE", unix.IN_CREATE},
		{"IN_DELETE", unix.IN_DELETE},
		{"IN_DELETE_SELF", unix.IN_DELETE_SELF},
		{"IN_DONT_FOLLOW", unix.IN_DONT_FOLLOW},
		{"IN_EXCL_UNLINK", unix.IN_EXCL_UNLINK},
		{"IN_IGNORED", unix.IN_IGNORED},
		{"IN_ISDIR", unix.IN_ISDIR},
		{"IN_LOOPBACKNET", unix.IN_LOOPBACKNET},
		{"IN_MASK_ADD", unix.IN_MASK_ADD},
		{"IN_MASK_CREATE", unix.IN_MASK_CREATE},
		{"IN_MODIFY", unix.IN_MODIFY},
		{"IN_MOVE", unix.IN_MOVE},
		{"IN_MOVED_FROM", unix.IN_MOVED_FROM},
		{"IN_MOVED_TO", unix.IN_MOVED_TO},
		{"IN_MOVE_SELF", unix.IN_MOVE_SELF},
		{"IN_ONESHOT", unix.IN_ONESHOT},
		{"IN_ONLYDIR", unix.IN_ONLYDIR},
		{"IN_OPEN", unix.IN_OPEN},
		{"IN_Q_OVERFLOW", unix.IN_Q_OVERFLOW},
		{"IN_UNMOUNT", unix.IN_UNMOUNT},
	}

	var l []string
	for _, n := range names {
		if mask&n.m == n.m {
			l = append(l, n.n)
		}
	}
	fmt.Fprintf(os.Stderr, "%s  %-20s → %s\n", time.Now().Format("15:04:05.0000"), strings.Join(l, " | "), name)
}

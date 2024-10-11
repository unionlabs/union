package main

import (
	"bytes"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"

	"go.etcd.io/bbolt/internal/common"
	"go.etcd.io/bbolt/internal/guts_cli"
)

// pageCommand represents the "page" command execution.
type pageCommand struct {
	baseCommand
}

// newPageCommand returns a pageCommand.
func newPageCommand(m *Main) *pageCommand {
	c := &pageCommand{}
	c.baseCommand = m.baseCommand
	return c
}

// Run executes the command.
func (cmd *pageCommand) Run(args ...string) error {
	// Parse flags.
	fs := flag.NewFlagSet("", flag.ContinueOnError)
	help := fs.Bool("h", false, "")
	all := fs.Bool("all", false, "list all pages")
	formatValue := fs.String("format-value", "auto", "One of: "+FORMAT_MODES+" . Applies to values on the leaf page.")

	if err := fs.Parse(args); err != nil {
		return err
	} else if *help {
		fmt.Fprintln(cmd.Stderr, cmd.Usage())
		return ErrUsage
	}

	// Require database path and page id.
	path := fs.Arg(0)
	if path == "" {
		return ErrPathRequired
	} else if _, err := os.Stat(path); os.IsNotExist(err) {
		return ErrFileNotFound
	}

	if !*all {
		// Read page ids.
		pageIDs, err := stringToPages(fs.Args()[1:])
		if err != nil {
			return err
		} else if len(pageIDs) == 0 {
			return ErrPageIDRequired
		}
		cmd.printPages(pageIDs, path, formatValue)
	} else {
		cmd.printAllPages(path, formatValue)
	}
	return nil
}

func (cmd *pageCommand) printPages(pageIDs []uint64, path string, formatValue *string) {
	// Print each page listed.
	for i, pageID := range pageIDs {
		// Print a separator.
		if i > 0 {
			fmt.Fprintln(cmd.Stdout, "===============================================")
		}
		_, err2 := cmd.printPage(path, pageID, *formatValue)
		if err2 != nil {
			fmt.Fprintf(cmd.Stdout, "Prining page %d failed: %s. Continuing...\n", pageID, err2)
		}
	}
}

func (cmd *pageCommand) printAllPages(path string, formatValue *string) {
	_, hwm, err := guts_cli.ReadPageAndHWMSize(path)
	if err != nil {
		fmt.Fprintf(cmd.Stdout, "cannot read number of pages: %v", err)
	}

	// Print each page listed.
	for pageID := uint64(0); pageID < uint64(hwm); {
		// Print a separator.
		if pageID > 0 {
			fmt.Fprintln(cmd.Stdout, "===============================================")
		}
		overflow, err2 := cmd.printPage(path, pageID, *formatValue)
		if err2 != nil {
			fmt.Fprintf(cmd.Stdout, "Prining page %d failed: %s. Continuing...\n", pageID, err2)
			pageID++
		} else {
			pageID += uint64(overflow) + 1
		}
	}
}

// printPage prints given page to cmd.Stdout and returns error or number of interpreted pages.
func (cmd *pageCommand) printPage(path string, pageID uint64, formatValue string) (numPages uint32, reterr error) {
	defer func() {
		if err := recover(); err != nil {
			reterr = fmt.Errorf("%s", err)
		}
	}()

	// Retrieve page info and page size.
	p, buf, err := guts_cli.ReadPage(path, pageID)
	if err != nil {
		return 0, err
	}

	// Print basic page info.
	fmt.Fprintf(cmd.Stdout, "Page ID:    %d\n", p.Id())
	fmt.Fprintf(cmd.Stdout, "Page Type:  %s\n", p.Typ())
	fmt.Fprintf(cmd.Stdout, "Total Size: %d bytes\n", len(buf))
	fmt.Fprintf(cmd.Stdout, "Overflow pages: %d\n", p.Overflow())

	// Print type-specific data.
	switch p.Typ() {
	case "meta":
		err = cmd.PrintMeta(cmd.Stdout, buf)
	case "leaf":
		err = cmd.PrintLeaf(cmd.Stdout, buf, formatValue)
	case "branch":
		err = cmd.PrintBranch(cmd.Stdout, buf)
	case "freelist":
		err = cmd.PrintFreelist(cmd.Stdout, buf)
	}
	if err != nil {
		return 0, err
	}
	return p.Overflow(), nil
}

// PrintMeta prints the data from the meta page.
func (cmd *pageCommand) PrintMeta(w io.Writer, buf []byte) error {
	m := common.LoadPageMeta(buf)
	m.Print(w)
	return nil
}

// PrintLeaf prints the data for a leaf page.
func (cmd *pageCommand) PrintLeaf(w io.Writer, buf []byte, formatValue string) error {
	p := common.LoadPage(buf)

	// Print number of items.
	fmt.Fprintf(w, "Item Count: %d\n", p.Count())
	fmt.Fprintf(w, "\n")

	// Print each key/value.
	for i := uint16(0); i < p.Count(); i++ {
		e := p.LeafPageElement(i)

		// Format key as string.
		var k string
		if isPrintable(string(e.Key())) {
			k = fmt.Sprintf("%q", string(e.Key()))
		} else {
			k = fmt.Sprintf("%x", string(e.Key()))
		}

		// Format value as string.
		var v string
		if e.IsBucketEntry() {
			b := e.Bucket()
			v = b.String()
		} else {
			var err error
			v, err = formatBytes(e.Value(), formatValue)
			if err != nil {
				return err
			}
		}

		fmt.Fprintf(w, "%s: %s\n", k, v)
	}
	fmt.Fprintf(w, "\n")
	return nil
}

// PrintBranch prints the data for a leaf page.
func (cmd *pageCommand) PrintBranch(w io.Writer, buf []byte) error {
	p := common.LoadPage(buf)

	// Print number of items.
	fmt.Fprintf(w, "Item Count: %d\n", p.Count())
	fmt.Fprintf(w, "\n")

	// Print each key/value.
	for i := uint16(0); i < p.Count(); i++ {
		e := p.BranchPageElement(i)

		// Format key as string.
		var k string
		if isPrintable(string(e.Key())) {
			k = fmt.Sprintf("%q", string(e.Key()))
		} else {
			k = fmt.Sprintf("%x", string(e.Key()))
		}

		fmt.Fprintf(w, "%s: <pgid=%d>\n", k, e.Pgid())
	}
	fmt.Fprintf(w, "\n")
	return nil
}

// PrintFreelist prints the data for a freelist page.
func (cmd *pageCommand) PrintFreelist(w io.Writer, buf []byte) error {
	p := common.LoadPage(buf)

	// Print number of items.
	_, cnt := p.FreelistPageCount()
	fmt.Fprintf(w, "Item Count: %d\n", cnt)
	fmt.Fprintf(w, "Overflow: %d\n", p.Overflow())

	fmt.Fprintf(w, "\n")

	// Print each page in the freelist.
	ids := p.FreelistPageIds()
	for _, ids := range ids {
		fmt.Fprintf(w, "%d\n", ids)
	}
	fmt.Fprintf(w, "\n")
	return nil
}

// PrintPage prints a given page as hexadecimal.
func (cmd *pageCommand) PrintPage(w io.Writer, r io.ReaderAt, pageID int, pageSize int) error {
	const bytesPerLineN = 16

	// Read page into buffer.
	buf := make([]byte, pageSize)
	addr := pageID * pageSize
	if n, err := r.ReadAt(buf, int64(addr)); err != nil {
		return err
	} else if n != pageSize {
		return io.ErrUnexpectedEOF
	}

	// Write out to writer in 16-byte lines.
	var prev []byte
	var skipped bool
	for offset := 0; offset < pageSize; offset += bytesPerLineN {
		// Retrieve current 16-byte line.
		line := buf[offset : offset+bytesPerLineN]
		isLastLine := offset == (pageSize - bytesPerLineN)

		// If it's the same as the previous line then print a skip.
		if bytes.Equal(line, prev) && !isLastLine {
			if !skipped {
				fmt.Fprintf(w, "%07x *\n", addr+offset)
				skipped = true
			}
		} else {
			// Print line as hexadecimal in 2-byte groups.
			fmt.Fprintf(w, "%07x %04x %04x %04x %04x %04x %04x %04x %04x\n", addr+offset,
				line[0:2], line[2:4], line[4:6], line[6:8],
				line[8:10], line[10:12], line[12:14], line[14:16],
			)

			skipped = false
		}

		// Save the previous line.
		prev = line
	}
	fmt.Fprint(w, "\n")

	return nil
}

// Usage returns the help message.
func (cmd *pageCommand) Usage() string {
	return strings.TrimLeft(`
usage: bolt page PATH pageid [pageid...]
   or: bolt page --all PATH

Additional options include:

	--all
		prints all pages (only skips pages that were considered successful overflow pages) 
	--format-value=`+FORMAT_MODES+` (default: auto)
		prints values (on the leaf page) using the given format.

Page prints one or more pages in human readable format.
`, "\n")
}

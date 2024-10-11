package replay

import (
	"bytes"
	"crypto/rand"
	"fmt"
	"io"
	"strconv"
	"strings"
	"testing"
	"time"
	"unicode"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func TestWorkloadCollector(t *testing.T) {
	const srcDir = `src`
	const destDir = `dst`
	datadriven.Walk(t, "testdata/collect", func(t *testing.T, path string) {
		fs := vfs.NewMem()
		require.NoError(t, fs.MkdirAll(srcDir, 0755))
		require.NoError(t, fs.MkdirAll(destDir, 0755))
		c := NewWorkloadCollector(srcDir)
		o := &pebble.Options{FS: fs}
		c.Attach(o)
		var currentManifest vfs.File
		var buf bytes.Buffer
		defer func() {
			if currentManifest != nil {
				currentManifest.Close()
			}
		}()
		datadriven.RunTest(t, path, func(t *testing.T, td *datadriven.TestData) string {
			buf.Reset()
			switch td.Cmd {
			case "cmp-files":
				if len(td.CmdArgs) != 2 {
					return fmt.Sprintf("expected exactly 2 args, received %d", len(td.CmdArgs))
				}
				b1 := readFile(t, fs, td.CmdArgs[0].String())
				b2 := readFile(t, fs, td.CmdArgs[1].String())
				if !bytes.Equal(b1, b2) {
					return fmt.Sprintf("files are unequal: %s (%s) and %s (%s)",
						td.CmdArgs[0].String(), humanize.Bytes.Uint64(uint64(len(b1))),
						td.CmdArgs[1].String(), humanize.Bytes.Uint64(uint64(len(b2))))
				}
				return "equal"
			case "clean":
				for _, path := range strings.Fields(td.Input) {
					typ, _, ok := base.ParseFilename(fs, path)
					require.True(t, ok)
					require.NoError(t, o.Cleaner.Clean(fs, typ, path))
				}
				return ""
			case "create-manifest":
				if currentManifest != nil {
					require.NoError(t, currentManifest.Close())
				}

				var fileNum uint64
				var err error
				td.ScanArgs(t, "filenum", &fileNum)
				path := base.MakeFilepath(fs, srcDir, base.FileTypeManifest, base.FileNum(fileNum).DiskFileNum())
				currentManifest, err = fs.Create(path)
				require.NoError(t, err)
				_, err = currentManifest.Write(randData(100))
				require.NoError(t, err)

				c.onManifestCreated(pebble.ManifestCreateInfo{
					Path:    path,
					FileNum: base.FileNum(fileNum),
				})
				return ""
			case "flush":
				flushInfo := pebble.FlushInfo{
					Done:          true,
					Input:         1,
					Duration:      100 * time.Millisecond,
					TotalDuration: 100 * time.Millisecond,
				}
				for _, line := range strings.Split(td.Input, "\n") {
					if line == "" {
						continue
					}

					parts := strings.FieldsFunc(line, func(r rune) bool { return unicode.IsSpace(r) || r == ':' })
					tableInfo := pebble.TableInfo{Size: 10 << 10}
					fileNum, err := strconv.ParseUint(parts[0], 10, 64)
					require.NoError(t, err)
					tableInfo.FileNum = base.FileNum(fileNum)

					p := writeFile(t, fs, srcDir, base.FileTypeTable, tableInfo.FileNum.DiskFileNum(), randData(int(tableInfo.Size)))
					fmt.Fprintf(&buf, "created %s\n", p)
					flushInfo.Output = append(flushInfo.Output, tableInfo)

					// Simulate a version edit applied to the current manifest.
					_, err = currentManifest.Write(randData(25))
					require.NoError(t, err)
				}
				flushInfo.InputBytes = 100 // Determinism
				fmt.Fprint(&buf, flushInfo.String())
				c.onFlushEnd(flushInfo)
				return buf.String()
			case "ingest":
				ingestInfo := pebble.TableIngestInfo{}
				for _, line := range strings.Split(td.Input, "\n") {
					if line == "" {
						continue
					}

					parts := strings.FieldsFunc(line, func(r rune) bool { return unicode.IsSpace(r) || r == ':' })
					tableInfo := pebble.TableInfo{Size: 10 << 10}
					fileNum, err := strconv.ParseUint(parts[0], 10, 64)
					require.NoError(t, err)
					tableInfo.FileNum = base.FileNum(fileNum)

					p := writeFile(t, fs, srcDir, base.FileTypeTable, tableInfo.FileNum.DiskFileNum(), randData(int(tableInfo.Size)))
					fmt.Fprintf(&buf, "created %s\n", p)
					ingestInfo.Tables = append(ingestInfo.Tables, struct {
						pebble.TableInfo
						Level int
					}{Level: 0, TableInfo: tableInfo})

					// Simulate a version edit applied to the current manifest.
					_, err = currentManifest.Write(randData(25))
					require.NoError(t, err)
				}
				fmt.Fprint(&buf, ingestInfo.String())
				c.onTableIngest(ingestInfo)
				return buf.String()

			case "ls":
				return runListFiles(t, fs, td)
			case "start":
				c.Start(fs, destDir)
				return ""
			case "stat":
				var buf bytes.Buffer
				for _, arg := range td.CmdArgs {
					fi, err := fs.Stat(arg.String())
					if err != nil {
						fmt.Fprintf(&buf, "%s: %s\n", arg.String(), err)
						continue
					}
					fmt.Fprintf(&buf, "%s:\n", arg.String())
					fmt.Fprintf(&buf, "  size: %d\n", fi.Size())
				}
				return buf.String()
			case "stop":
				c.Stop()
				return ""
			case "wait":
				// Wait until all pending sstables have been copied, then list
				// the files in the destination directory.
				c.mu.Lock()
				for c.mu.tablesEnqueued != c.mu.tablesCopied {
					c.mu.copyCond.Wait()
				}
				c.mu.Unlock()
				listFiles(t, fs, &buf, destDir)
				return buf.String()
			default:
				return fmt.Sprintf("unrecognized command %q", td.Cmd)
			}
		})
	})
}

func randData(byteCount int) []byte {
	b := make([]byte, byteCount)
	rand.Read(b)
	return b
}

func writeFile(
	t *testing.T, fs vfs.FS, dir string, typ base.FileType, fileNum base.DiskFileNum, data []byte,
) string {
	path := base.MakeFilepath(fs, dir, typ, fileNum)
	f, err := fs.Create(path)
	require.NoError(t, err)
	_, err = f.Write(data)
	require.NoError(t, err)
	require.NoError(t, f.Close())
	return path
}

func readFile(t *testing.T, fs vfs.FS, path string) []byte {
	r, err := fs.Open(path)
	require.NoError(t, err)
	b, err := io.ReadAll(r)
	require.NoError(t, err)
	require.NoError(t, r.Close())
	return b
}

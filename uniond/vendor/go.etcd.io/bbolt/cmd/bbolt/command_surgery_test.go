package main_test

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	bolt "go.etcd.io/bbolt"
	main "go.etcd.io/bbolt/cmd/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
	"go.etcd.io/bbolt/internal/common"
	"go.etcd.io/bbolt/internal/guts_cli"
)

func TestSurgery_RevertMetaPage(t *testing.T) {
	pageSize := 4096
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
	srcPath := db.Path()

	defer requireDBNoChange(t, dbData(t, db.Path()), db.Path())

	srcFile, err := os.Open(srcPath)
	require.NoError(t, err)
	defer srcFile.Close()

	// Read both meta0 and meta1 from srcFile
	srcBuf0 := readPage(t, srcPath, 0, pageSize)
	srcBuf1 := readPage(t, srcPath, 1, pageSize)
	meta0Page := common.LoadPageMeta(srcBuf0)
	meta1Page := common.LoadPageMeta(srcBuf1)

	// Get the non-active meta page
	nonActiveSrcBuf := srcBuf0
	nonActiveMetaPageId := 0
	if meta0Page.Txid() > meta1Page.Txid() {
		nonActiveSrcBuf = srcBuf1
		nonActiveMetaPageId = 1
	}
	t.Logf("non active meta page id: %d", nonActiveMetaPageId)

	// revert the meta page
	rootCmd := main.NewRootCommand()
	output := filepath.Join(t.TempDir(), "db")
	rootCmd.SetArgs([]string{
		"surgery", "revert-meta-page", srcPath,
		"--output", output,
	})
	err = rootCmd.Execute()
	require.NoError(t, err)

	// read both meta0 and meta1 from dst file
	dstBuf0 := readPage(t, output, 0, pageSize)
	dstBuf1 := readPage(t, output, 1, pageSize)

	// check result. Note we should skip the page ID
	assert.Equal(t, pageDataWithoutPageId(nonActiveSrcBuf), pageDataWithoutPageId(dstBuf0))
	assert.Equal(t, pageDataWithoutPageId(nonActiveSrcBuf), pageDataWithoutPageId(dstBuf1))
}

func TestSurgery_CopyPage(t *testing.T) {
	pageSize := 4096
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
	srcPath := db.Path()

	// Insert some sample data
	t.Log("Insert some sample data")
	err := db.Fill([]byte("data"), 1, 20,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
		func(tx int, k int) []byte { return make([]byte, 10) },
	)
	require.NoError(t, err)

	defer requireDBNoChange(t, dbData(t, srcPath), srcPath)

	// copy page 3 to page 2
	t.Log("copy page 3 to page 2")
	rootCmd := main.NewRootCommand()
	output := filepath.Join(t.TempDir(), "dstdb")
	rootCmd.SetArgs([]string{
		"surgery", "copy-page", srcPath,
		"--output", output,
		"--from-page", "3",
		"--to-page", "2",
	})
	err = rootCmd.Execute()
	require.NoError(t, err)

	// The page 2 should have exactly the same data as page 3.
	t.Log("Verify result")
	srcPageId3Data := readPage(t, srcPath, 3, pageSize)
	dstPageId3Data := readPage(t, output, 3, pageSize)
	dstPageId2Data := readPage(t, output, 2, pageSize)

	assert.Equal(t, srcPageId3Data, dstPageId3Data)
	assert.Equal(t, pageDataWithoutPageId(srcPageId3Data), pageDataWithoutPageId(dstPageId2Data))
}

// TODO(ahrtr): add test case below for `surgery clear-page` command:
//  1. The page is a branch page. All its children should become free pages.
func TestSurgery_ClearPage(t *testing.T) {
	pageSize := 4096
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
	srcPath := db.Path()

	// Insert some sample data
	t.Log("Insert some sample data")
	err := db.Fill([]byte("data"), 1, 20,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
		func(tx int, k int) []byte { return make([]byte, 10) },
	)
	require.NoError(t, err)

	defer requireDBNoChange(t, dbData(t, srcPath), srcPath)

	// clear page 3
	t.Log("clear page 3")
	rootCmd := main.NewRootCommand()
	output := filepath.Join(t.TempDir(), "dstdb")
	rootCmd.SetArgs([]string{
		"surgery", "clear-page", srcPath,
		"--output", output,
		"--pageId", "3",
	})
	err = rootCmd.Execute()
	require.NoError(t, err)

	t.Log("Verify result")
	dstPageId3Data := readPage(t, output, 3, pageSize)

	p := common.LoadPage(dstPageId3Data)
	assert.Equal(t, uint16(0), p.Count())
	assert.Equal(t, uint32(0), p.Overflow())
}

func TestSurgery_ClearPageElements_Without_Overflow(t *testing.T) {
	testCases := []struct {
		name                 string
		from                 int
		to                   int
		isBranchPage         bool
		setEndIdxAsCount     bool
		removeOnlyOneElement bool // only valid when setEndIdxAsCount == true, and startIdx = endIdx -1 in this case.
		expectError          bool
	}{
		// normal range in leaf page
		{
			name: "normal range in leaf page: [4, 8)",
			from: 4,
			to:   8,
		},
		{
			name: "normal range in leaf page: [5, -1)",
			from: 4,
			to:   -1,
		},
		{
			name: "normal range in leaf page: all",
			from: 0,
			to:   -1,
		},
		{
			name: "normal range in leaf page: [0, 7)",
			from: 0,
			to:   7,
		},
		{
			name:             "normal range in leaf page: [3, count)",
			from:             4,
			setEndIdxAsCount: true,
		},
		// normal range in branch page
		{
			name:         "normal range in branch page: [4, 8)",
			from:         4,
			to:           8,
			isBranchPage: true,
		},
		{
			name:         "normal range in branch page: [5, -1)",
			from:         4,
			to:           -1,
			isBranchPage: true,
		},
		{
			name:         "normal range in branch page: all",
			from:         0,
			to:           -1,
			isBranchPage: true,
		},
		{
			name:         "normal range in branch page: [0, 7)",
			from:         0,
			to:           7,
			isBranchPage: true,
		},
		{
			name:             "normal range in branch page: [3, count)",
			from:             4,
			isBranchPage:     true,
			setEndIdxAsCount: true,
		},
		// remove only one element
		{
			name: "one element: the first one",
			from: 0,
			to:   1,
		},
		{
			name: "one element: [6, 7)",
			from: 6,
			to:   7,
		},
		{
			name:                 "one element: the last one",
			setEndIdxAsCount:     true,
			removeOnlyOneElement: true,
		},
		// abnormal range
		{
			name:        "abnormal range: [-1, 4)",
			from:        -1,
			to:          4,
			expectError: true,
		},
		{
			name:        "abnormal range: [-2, 5)",
			from:        -1,
			to:          5,
			expectError: true,
		},
		{
			name:        "abnormal range: [3, 3)",
			from:        3,
			to:          3,
			expectError: true,
		},
		{
			name:        "abnormal range: [5, 3)",
			from:        5,
			to:          3,
			expectError: true,
		},
		{
			name:        "abnormal range: [3, -2)",
			from:        3,
			to:          -2,
			expectError: true,
		},
		{
			name:        "abnormal range: [3, 1000000)",
			from:        -1,
			to:          4,
			expectError: true,
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			testSurgeryClearPageElementsWithoutOverflow(t, tc.from, tc.to, tc.isBranchPage, tc.setEndIdxAsCount, tc.removeOnlyOneElement, tc.expectError)
		})
	}
}

func testSurgeryClearPageElementsWithoutOverflow(t *testing.T, startIdx, endIdx int, isBranchPage, setEndIdxAsCount, removeOnlyOne, expectError bool) {
	pageSize := 4096
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
	srcPath := db.Path()

	// Generate sample db
	t.Log("Generate some sample data")
	err := db.Fill([]byte("data"), 10, 200,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", tx*10000+k)) },
		func(tx int, k int) []byte { return make([]byte, 10) },
	)
	require.NoError(t, err)

	defer requireDBNoChange(t, dbData(t, srcPath), srcPath)

	// find a page with at least 10 elements
	var (
		pageId       uint64 = 2
		elementCount uint16 = 0
	)
	for {
		p, _, err := guts_cli.ReadPage(srcPath, pageId)
		require.NoError(t, err)

		if isBranchPage {
			if p.IsBranchPage() && p.Count() > 10 {
				elementCount = p.Count()
				break
			}
		} else {
			if p.IsLeafPage() && p.Count() > 10 {
				elementCount = p.Count()
				break
			}
		}
		pageId++
	}
	t.Logf("The original element count: %d", elementCount)

	if setEndIdxAsCount {
		t.Logf("Set the endIdx as the element count: %d", elementCount)
		endIdx = int(elementCount)
		if removeOnlyOne {
			startIdx = endIdx - 1
			t.Logf("Set the startIdx as the endIdx-1: %d", startIdx)
		}
	}

	// clear elements [startIdx, endIdx) in the page
	rootCmd := main.NewRootCommand()
	output := filepath.Join(t.TempDir(), "db")
	rootCmd.SetArgs([]string{
		"surgery", "clear-page-elements", srcPath,
		"--output", output,
		"--pageId", fmt.Sprintf("%d", pageId),
		"--from-index", fmt.Sprintf("%d", startIdx),
		"--to-index", fmt.Sprintf("%d", endIdx),
	})
	err = rootCmd.Execute()
	if expectError {
		require.Error(t, err)
		return
	}

	require.NoError(t, err)

	// check the element count again
	expectedCnt := 0
	if endIdx == -1 {
		expectedCnt = startIdx
	} else {
		expectedCnt = int(elementCount) - (endIdx - startIdx)
	}
	p, _, err := guts_cli.ReadPage(output, pageId)
	require.NoError(t, err)
	assert.Equal(t, expectedCnt, int(p.Count()))

	compareDataAfterClearingElement(t, srcPath, output, pageId, isBranchPage, startIdx, endIdx)
}

func compareDataAfterClearingElement(t *testing.T, srcPath, dstPath string, pageId uint64, isBranchPage bool, startIdx, endIdx int) {
	srcPage, _, err := guts_cli.ReadPage(srcPath, pageId)
	require.NoError(t, err)

	dstPage, _, err := guts_cli.ReadPage(dstPath, pageId)
	require.NoError(t, err)

	var dstIdx uint16
	for i := uint16(0); i < srcPage.Count(); i++ {
		// skip the cleared elements
		if dstIdx >= uint16(startIdx) && (dstIdx < uint16(endIdx) || endIdx == -1) {
			continue
		}

		if isBranchPage {
			srcElement := srcPage.BranchPageElement(i)
			dstElement := dstPage.BranchPageElement(dstIdx)

			require.Equal(t, srcElement.Key(), dstElement.Key())
			require.Equal(t, srcElement.Pgid(), dstElement.Pgid())
		} else {
			srcElement := srcPage.LeafPageElement(i)
			dstElement := dstPage.LeafPageElement(dstIdx)

			require.Equal(t, srcElement.Flags(), dstElement.Flags())
			require.Equal(t, srcElement.Key(), dstElement.Key())
			require.Equal(t, srcElement.Value(), dstElement.Value())
		}

		dstIdx++
	}
}

func TestSurgery_ClearPageElements_With_Overflow(t *testing.T) {
	testCases := []struct {
		name             string
		from             int
		to               int
		valueSizes       []int
		expectedOverflow int
	}{
		// big element
		{
			name:             "remove a big element at the end",
			valueSizes:       []int{500, 500, 500, 2600},
			from:             3,
			to:               4,
			expectedOverflow: 0,
		},
		{
			name:             "remove a big element at the begin",
			valueSizes:       []int{2600, 500, 500, 500},
			from:             0,
			to:               1,
			expectedOverflow: 0,
		},
		{
			name:             "remove a big element in the middle",
			valueSizes:       []int{500, 2600, 500, 500},
			from:             1,
			to:               2,
			expectedOverflow: 0,
		},
		// small element
		{
			name:             "remove a small element at the end",
			valueSizes:       []int{500, 500, 3100, 100},
			from:             3,
			to:               4,
			expectedOverflow: 1,
		},
		{
			name:             "remove a small element at the begin",
			valueSizes:       []int{100, 500, 3100, 500},
			from:             0,
			to:               1,
			expectedOverflow: 1,
		},
		{
			name:             "remove a small element in the middle",
			valueSizes:       []int{500, 100, 3100, 500},
			from:             1,
			to:               2,
			expectedOverflow: 1,
		},
		{
			name:             "remove a small element at the end of page with big overflow",
			valueSizes:       []int{500, 500, 4096 * 5, 100},
			from:             3,
			to:               4,
			expectedOverflow: 5,
		},
		{
			name:             "remove a small element at the begin of page with big overflow",
			valueSizes:       []int{100, 500, 4096 * 6, 500},
			from:             0,
			to:               1,
			expectedOverflow: 6,
		},
		{
			name:             "remove a small element in the middle of page with big overflow",
			valueSizes:       []int{500, 100, 4096 * 4, 500},
			from:             1,
			to:               2,
			expectedOverflow: 4,
		},
		// huge element
		{
			name:             "remove a huge element at the end",
			valueSizes:       []int{500, 500, 500, 4096 * 5},
			from:             3,
			to:               4,
			expectedOverflow: 0,
		},
		{
			name:             "remove a huge element at the begin",
			valueSizes:       []int{4096 * 5, 500, 500, 500},
			from:             0,
			to:               1,
			expectedOverflow: 0,
		},
		{
			name:             "remove a huge element in the middle",
			valueSizes:       []int{500, 4096 * 5, 500, 500},
			from:             1,
			to:               2,
			expectedOverflow: 0,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			testSurgeryClearPageElementsWithOverflow(t, tc.from, tc.to, tc.valueSizes, tc.expectedOverflow)
		})
	}
}

func testSurgeryClearPageElementsWithOverflow(t *testing.T, startIdx, endIdx int, valueSizes []int, expectedOverflow int) {
	pageSize := 4096
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
	srcPath := db.Path()

	// Generate sample db
	err := db.Update(func(tx *bolt.Tx) error {
		b, _ := tx.CreateBucketIfNotExists([]byte("data"))
		for i, valueSize := range valueSizes {
			key := []byte(fmt.Sprintf("%04d", i))
			val := make([]byte, valueSize)
			if putErr := b.Put(key, val); putErr != nil {
				return putErr
			}
		}
		return nil
	})
	require.NoError(t, err)

	defer requireDBNoChange(t, dbData(t, srcPath), srcPath)

	// find a page with overflow pages
	var (
		pageId       uint64 = 2
		elementCount uint16 = 0
	)
	for {
		p, _, err := guts_cli.ReadPage(srcPath, pageId)
		require.NoError(t, err)

		if p.Overflow() > 0 {
			elementCount = p.Count()
			break
		}
		pageId++
	}
	t.Logf("The original element count: %d", elementCount)

	// clear elements [startIdx, endIdx) in the page
	rootCmd := main.NewRootCommand()
	output := filepath.Join(t.TempDir(), "db")
	rootCmd.SetArgs([]string{
		"surgery", "clear-page-elements", srcPath,
		"--output", output,
		"--pageId", fmt.Sprintf("%d", pageId),
		"--from-index", fmt.Sprintf("%d", startIdx),
		"--to-index", fmt.Sprintf("%d", endIdx),
	})
	err = rootCmd.Execute()
	require.NoError(t, err)

	// check the element count again
	expectedCnt := 0
	if endIdx == -1 {
		expectedCnt = startIdx
	} else {
		expectedCnt = int(elementCount) - (endIdx - startIdx)
	}
	p, _, err := guts_cli.ReadPage(output, pageId)
	require.NoError(t, err)
	assert.Equal(t, expectedCnt, int(p.Count()))

	assert.Equal(t, expectedOverflow, int(p.Overflow()))

	compareDataAfterClearingElement(t, srcPath, output, pageId, false, startIdx, endIdx)
}

func TestSurgeryRequiredFlags(t *testing.T) {
	errMsgFmt := `required flag(s) "%s" not set`
	testCases := []struct {
		name           string
		args           []string
		expectedErrMsg string
	}{
		// --output is required for all surgery commands
		{
			name:           "no output flag for revert-meta-page",
			args:           []string{"surgery", "revert-meta-page", "db"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "output"),
		},
		{
			name:           "no output flag for copy-page",
			args:           []string{"surgery", "copy-page", "db", "--from-page", "3", "--to-page", "2"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "output"),
		},
		{
			name:           "no output flag for clear-page",
			args:           []string{"surgery", "clear-page", "db", "--pageId", "3"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "output"),
		},
		{
			name:           "no output flag for clear-page-element",
			args:           []string{"surgery", "clear-page-elements", "db", "--pageId", "4", "--from-index", "3", "--to-index", "5"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "output"),
		},
		{
			name:           "no output flag for freelist abandon",
			args:           []string{"surgery", "freelist", "abandon", "db"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "output"),
		},
		{
			name:           "no output flag for freelist rebuild",
			args:           []string{"surgery", "freelist", "rebuild", "db"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "output"),
		},
		// --from-page and --to-page are required for 'surgery copy-page' command
		{
			name:           "no from-page flag for copy-page",
			args:           []string{"surgery", "copy-page", "db", "--output", "db", "--to-page", "2"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "from-page"),
		},
		{
			name:           "no to-page flag for copy-page",
			args:           []string{"surgery", "copy-page", "db", "--output", "db", "--from-page", "2"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "to-page"),
		},
		// --pageId is required for 'surgery clear-page' command
		{
			name:           "no pageId flag for clear-page",
			args:           []string{"surgery", "clear-page", "db", "--output", "db"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "pageId"),
		},
		// --pageId, --from-index and --to-index are required for 'surgery clear-page-element' command
		{
			name:           "no pageId flag for clear-page-element",
			args:           []string{"surgery", "clear-page-elements", "db", "--output", "newdb", "--from-index", "3", "--to-index", "5"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "pageId"),
		},
		{
			name:           "no from-index flag for clear-page-element",
			args:           []string{"surgery", "clear-page-elements", "db", "--output", "newdb", "--pageId", "2", "--to-index", "5"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "from-index"),
		},
		{
			name:           "no to-index flag for clear-page-element",
			args:           []string{"surgery", "clear-page-elements", "db", "--output", "newdb", "--pageId", "2", "--from-index", "3"},
			expectedErrMsg: fmt.Sprintf(errMsgFmt, "to-index"),
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			rootCmd := main.NewRootCommand()
			rootCmd.SetArgs(tc.args)
			err := rootCmd.Execute()
			require.ErrorContains(t, err, tc.expectedErrMsg)
		})
	}
}

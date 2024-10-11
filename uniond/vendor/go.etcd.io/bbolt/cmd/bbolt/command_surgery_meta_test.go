package main_test

import (
	"fmt"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	bolt "go.etcd.io/bbolt"
	main "go.etcd.io/bbolt/cmd/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
	"go.etcd.io/bbolt/internal/common"
)

func TestSurgery_Meta_Validate(t *testing.T) {
	pageSize := 4096
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
	srcPath := db.Path()

	defer requireDBNoChange(t, dbData(t, db.Path()), db.Path())

	// validate the meta pages
	rootCmd := main.NewRootCommand()
	rootCmd.SetArgs([]string{
		"surgery", "meta", "validate", srcPath,
	})
	err := rootCmd.Execute()
	require.NoError(t, err)

	// TODD: add one more case that the validation may fail. We need to
	// make the command output configurable, so that test cases can set
	// a customized io.Writer.
}

func TestSurgery_Meta_Update(t *testing.T) {
	testCases := []struct {
		name     string
		root     common.Pgid
		freelist common.Pgid
		pgid     common.Pgid
	}{
		{
			name: "root changed",
			root: 50,
		},
		{
			name:     "freelist changed",
			freelist: 40,
		},
		{
			name: "pgid changed",
			pgid: 600,
		},
		{
			name:     "both root and freelist changed",
			root:     45,
			freelist: 46,
		},
		{
			name:     "both pgid and freelist changed",
			pgid:     256,
			freelist: 47,
		},
		{
			name:     "all fields changed",
			root:     43,
			freelist: 62,
			pgid:     256,
		},
	}

	for _, tc := range testCases {
		for i := 0; i <= 1; i++ {
			tc := tc
			metaPageId := uint32(i)

			t.Run(tc.name, func(t *testing.T) {
				pageSize := 4096
				db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
				srcPath := db.Path()

				defer requireDBNoChange(t, dbData(t, db.Path()), db.Path())

				var fields []string
				if tc.root != 0 {
					fields = append(fields, fmt.Sprintf("root:%d", tc.root))
				}
				if tc.freelist != 0 {
					fields = append(fields, fmt.Sprintf("freelist:%d", tc.freelist))
				}
				if tc.pgid != 0 {
					fields = append(fields, fmt.Sprintf("pgid:%d", tc.pgid))
				}

				rootCmd := main.NewRootCommand()
				output := filepath.Join(t.TempDir(), "db")
				rootCmd.SetArgs([]string{
					"surgery", "meta", "update", srcPath,
					"--output", output,
					"--meta-page", fmt.Sprintf("%d", metaPageId),
					"--fields", strings.Join(fields, ","),
				})
				err := rootCmd.Execute()
				require.NoError(t, err)

				m, _, err := main.ReadMetaPageAt(output, metaPageId, 4096)
				require.NoError(t, err)

				require.Equal(t, common.Magic, m.Magic())
				require.Equal(t, common.Version, m.Version())

				if tc.root != 0 {
					require.Equal(t, tc.root, m.RootBucket().RootPage())
				}
				if tc.freelist != 0 {
					require.Equal(t, tc.freelist, m.Freelist())
				}
				if tc.pgid != 0 {
					require.Equal(t, tc.pgid, m.Pgid())
				}
			})
		}
	}
}

package main_test

import (
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	bolt "go.etcd.io/bbolt"
	main "go.etcd.io/bbolt/cmd/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
	"go.etcd.io/bbolt/internal/common"
)

func TestSurgery_Freelist_Abandon(t *testing.T) {
	pageSize := 4096
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize})
	srcPath := db.Path()

	defer requireDBNoChange(t, dbData(t, srcPath), srcPath)

	rootCmd := main.NewRootCommand()
	output := filepath.Join(t.TempDir(), "db")
	rootCmd.SetArgs([]string{
		"surgery", "freelist", "abandon", srcPath,
		"--output", output,
	})
	err := rootCmd.Execute()
	require.NoError(t, err)

	meta0 := loadMetaPage(t, output, 0)
	assert.Equal(t, common.PgidNoFreelist, meta0.Freelist())
	meta1 := loadMetaPage(t, output, 1)
	assert.Equal(t, common.PgidNoFreelist, meta1.Freelist())
}

func TestSurgery_Freelist_Rebuild(t *testing.T) {
	testCases := []struct {
		name          string
		hasFreelist   bool
		expectedError error
	}{
		{
			name:          "normal operation",
			hasFreelist:   false,
			expectedError: nil,
		},
		{
			name:          "already has freelist",
			hasFreelist:   true,
			expectedError: main.ErrSurgeryFreelistAlreadyExist,
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			pageSize := 4096
			db := btesting.MustCreateDBWithOption(t, &bolt.Options{
				PageSize:       pageSize,
				NoFreelistSync: !tc.hasFreelist,
			})
			srcPath := db.Path()

			err := db.Update(func(tx *bolt.Tx) error {
				// do nothing
				return nil
			})
			require.NoError(t, err)

			defer requireDBNoChange(t, dbData(t, srcPath), srcPath)

			// Verify the freelist isn't synced in the beginning
			meta := readMetaPage(t, srcPath)
			if tc.hasFreelist {
				if meta.Freelist() <= 1 || meta.Freelist() >= meta.Pgid() {
					t.Fatalf("freelist (%d) isn't in the valid range (1, %d)", meta.Freelist(), meta.Pgid())
				}
			} else {
				require.Equal(t, common.PgidNoFreelist, meta.Freelist())
			}

			// Execute `surgery freelist rebuild` command
			rootCmd := main.NewRootCommand()
			output := filepath.Join(t.TempDir(), "db")
			rootCmd.SetArgs([]string{
				"surgery", "freelist", "rebuild", srcPath,
				"--output", output,
			})
			err = rootCmd.Execute()
			require.Equal(t, tc.expectedError, err)

			if tc.expectedError == nil {
				// Verify the freelist has already been rebuilt.
				meta = readMetaPage(t, output)
				if meta.Freelist() <= 1 || meta.Freelist() >= meta.Pgid() {
					t.Fatalf("freelist (%d) isn't in the valid range (1, %d)", meta.Freelist(), meta.Pgid())
				}
			}
		})
	}
}

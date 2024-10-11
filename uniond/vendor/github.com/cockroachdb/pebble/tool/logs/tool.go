// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package logs

import (
	"time"

	"github.com/spf13/cobra"
)

// NewCmd returns a new cobra.Command for parsing logs.
func NewCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "logs",
		Short: "Scan and summarize logs",
	}

	compactionCmd := &cobra.Command{
		Use:   "compactions",
		Short: "Scan and summarize compaction logs",
		RunE:  runCompactionLogs,
	}
	compactionCmd.Flags().Duration(
		"window", 10*time.Minute, "time window in which to aggregate compactions")
	compactionCmd.Flags().Duration(
		"long-running-limit", 0, "log compactions with runtime greater than the limit")

	cmd.AddCommand(compactionCmd)
	return cmd
}

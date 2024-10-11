package main

import (
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"

	"github.com/spf13/cobra"
	"github.com/spf13/pflag"

	"go.etcd.io/bbolt/internal/common"
)

const (
	metaFieldPageSize = "pageSize"
	metaFieldRoot     = "root"
	metaFieldFreelist = "freelist"
	metaFieldPgid     = "pgid"
)

func newSurgeryMetaCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "meta <subcommand>",
		Short: "meta page related surgery commands",
	}

	cmd.AddCommand(newSurgeryMetaValidateCommand())
	cmd.AddCommand(newSurgeryMetaUpdateCommand())

	return cmd
}

func newSurgeryMetaValidateCommand() *cobra.Command {
	metaValidateCmd := &cobra.Command{
		Use:   "validate <bbolt-file>",
		Short: "Validate both meta pages",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			return surgeryMetaValidateFunc(args[0])
		},
	}
	return metaValidateCmd
}

func surgeryMetaValidateFunc(srcDBPath string) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	var pageSize uint32

	for i := 0; i <= 1; i++ {
		m, _, err := ReadMetaPageAt(srcDBPath, uint32(i), pageSize)
		if err != nil {
			return fmt.Errorf("read meta page %d failed: %w", i, err)
		}
		if mValidateErr := m.Validate(); mValidateErr != nil {
			fmt.Fprintf(os.Stdout, "WARNING: The meta page %d isn't valid: %v!\n", i, mValidateErr)
		} else {
			fmt.Fprintf(os.Stdout, "The meta page %d is valid!\n", i)
		}

		pageSize = m.PageSize()
	}

	return nil
}

type surgeryMetaUpdateOptions struct {
	surgeryBaseOptions
	fields     []string
	metaPageId uint32
}

var allowedMetaUpdateFields = map[string]struct{}{
	metaFieldPageSize: {},
	metaFieldRoot:     {},
	metaFieldFreelist: {},
	metaFieldPgid:     {},
}

// AddFlags sets the flags for `meta update` command.
// Example: --fields root:16,freelist:8 --fields pgid:128
// Result: []string{"root:16", "freelist:8", "pgid:128"}
func (o *surgeryMetaUpdateOptions) AddFlags(fs *pflag.FlagSet) {
	o.surgeryBaseOptions.AddFlags(fs)
	fs.StringSliceVarP(&o.fields, "fields", "", o.fields, "comma separated list of fields (supported fields: pageSize, root, freelist and pgid) to be updated, and each item is a colon-separated key-value pair")
	fs.Uint32VarP(&o.metaPageId, "meta-page", "", o.metaPageId, "the meta page ID to operate on, valid values are 0 and 1")
}

func (o *surgeryMetaUpdateOptions) Validate() error {
	if err := o.surgeryBaseOptions.Validate(); err != nil {
		return err
	}

	if o.metaPageId > 1 {
		return fmt.Errorf("invalid meta page id: %d", o.metaPageId)
	}

	for _, field := range o.fields {
		kv := strings.Split(field, ":")
		if len(kv) != 2 {
			return fmt.Errorf("invalid key-value pair: %s", field)
		}

		if _, ok := allowedMetaUpdateFields[kv[0]]; !ok {
			return fmt.Errorf("field %q isn't allowed to be updated", kv[0])
		}

		if _, err := strconv.ParseUint(kv[1], 10, 64); err != nil {
			return fmt.Errorf("invalid value %q for field %q", kv[1], kv[0])
		}
	}

	return nil
}

func newSurgeryMetaUpdateCommand() *cobra.Command {
	var o surgeryMetaUpdateOptions
	metaUpdateCmd := &cobra.Command{
		Use:   "update <bbolt-file>",
		Short: "Update fields in meta pages",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := o.Validate(); err != nil {
				return err
			}
			return surgeryMetaUpdateFunc(args[0], o)
		},
	}
	o.AddFlags(metaUpdateCmd.Flags())
	return metaUpdateCmd
}

func surgeryMetaUpdateFunc(srcDBPath string, cfg surgeryMetaUpdateOptions) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	if err := common.CopyFile(srcDBPath, cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("[meta update] copy file failed: %w", err)
	}

	// read the page size from the first meta page if we want to edit the second meta page.
	var pageSize uint32
	if cfg.metaPageId == 1 {
		m0, _, err := ReadMetaPageAt(cfg.outputDBFilePath, 0, pageSize)
		if err != nil {
			return fmt.Errorf("read the first meta page failed: %w", err)
		}
		pageSize = m0.PageSize()
	}

	// update the specified meta page
	m, buf, err := ReadMetaPageAt(cfg.outputDBFilePath, cfg.metaPageId, pageSize)
	if err != nil {
		return fmt.Errorf("read meta page %d failed: %w", cfg.metaPageId, err)
	}
	mChanged := updateMetaField(m, parseFields(cfg.fields))
	if mChanged {
		if err := writeMetaPageAt(cfg.outputDBFilePath, buf, cfg.metaPageId, pageSize); err != nil {
			return fmt.Errorf("[meta update] write meta page %d failed: %w", cfg.metaPageId, err)
		}
	}

	if cfg.metaPageId == 1 && pageSize != m.PageSize() {
		fmt.Fprintf(os.Stdout, "WARNING: The page size (%d) in the first meta page doesn't match the second meta page (%d)\n", pageSize, m.PageSize())
	}

	// Display results
	if !mChanged {
		fmt.Fprintln(os.Stdout, "Nothing changed!")
	}

	if mChanged {
		fmt.Fprintf(os.Stdout, "The meta page %d has been updated!\n", cfg.metaPageId)
	}

	return nil
}

func parseFields(fields []string) map[string]uint64 {
	fieldsMap := make(map[string]uint64)
	for _, field := range fields {
		kv := strings.SplitN(field, ":", 2)
		val, _ := strconv.ParseUint(kv[1], 10, 64)
		fieldsMap[kv[0]] = val
	}
	return fieldsMap
}

func updateMetaField(m *common.Meta, fields map[string]uint64) bool {
	changed := false
	for key, val := range fields {
		switch key {
		case metaFieldPageSize:
			m.SetPageSize(uint32(val))
		case metaFieldRoot:
			m.SetRootBucket(common.NewInBucket(common.Pgid(val), 0))
		case metaFieldFreelist:
			m.SetFreelist(common.Pgid(val))
		case metaFieldPgid:
			m.SetPgid(common.Pgid(val))
		}

		changed = true
	}

	if m.Magic() != common.Magic {
		m.SetMagic(common.Magic)
		changed = true
	}
	if m.Version() != common.Version {
		m.SetVersion(common.Version)
		changed = true
	}
	if m.Flags() != common.MetaPageFlag {
		m.SetFlags(common.MetaPageFlag)
		changed = true
	}

	newChecksum := m.Sum64()
	if m.Checksum() != newChecksum {
		m.SetChecksum(newChecksum)
		changed = true
	}

	return changed
}

func ReadMetaPageAt(dbPath string, metaPageId uint32, pageSize uint32) (*common.Meta, []byte, error) {
	if metaPageId > 1 {
		return nil, nil, fmt.Errorf("invalid metaPageId: %d", metaPageId)
	}

	f, err := os.OpenFile(dbPath, os.O_RDONLY, 0444)
	if err != nil {
		return nil, nil, err
	}
	defer f.Close()

	// The meta page is just 64 bytes, and definitely less than 1024 bytes,
	// so it's fine to only read 1024 bytes. Note we don't care about the
	// pageSize when reading the first meta page, because we always read the
	// file starting from offset 0. Actually the passed pageSize is 0 when
	// reading the first meta page in the `surgery meta update` command.
	buf := make([]byte, 1024)
	n, err := f.ReadAt(buf, int64(metaPageId*pageSize))
	if n == len(buf) && (err == nil || err == io.EOF) {
		return common.LoadPageMeta(buf), buf, nil
	}

	return nil, nil, err
}

func writeMetaPageAt(dbPath string, buf []byte, metaPageId uint32, pageSize uint32) error {
	if metaPageId > 1 {
		return fmt.Errorf("invalid metaPageId: %d", metaPageId)
	}

	f, err := os.OpenFile(dbPath, os.O_RDWR, 0666)
	if err != nil {
		return err
	}
	defer f.Close()

	n, err := f.WriteAt(buf, int64(metaPageId*pageSize))
	if n == len(buf) && (err == nil || err == io.EOF) {
		return nil
	}

	return err
}

package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/spf13/pflag"

	"go.etcd.io/bbolt/internal/common"
	"go.etcd.io/bbolt/internal/guts_cli"
	"go.etcd.io/bbolt/internal/surgeon"
)

var (
	ErrSurgeryFreelistAlreadyExist = errors.New("the file already has freelist, please consider to abandon the freelist to forcibly rebuild it")
)

func newSurgeryCommand() *cobra.Command {
	surgeryCmd := &cobra.Command{
		Use:   "surgery <subcommand>",
		Short: "surgery related commands",
	}

	surgeryCmd.AddCommand(newSurgeryRevertMetaPageCommand())
	surgeryCmd.AddCommand(newSurgeryCopyPageCommand())
	surgeryCmd.AddCommand(newSurgeryClearPageCommand())
	surgeryCmd.AddCommand(newSurgeryClearPageElementsCommand())
	surgeryCmd.AddCommand(newSurgeryFreelistCommand())
	surgeryCmd.AddCommand(newSurgeryMetaCommand())

	return surgeryCmd
}

type surgeryBaseOptions struct {
	outputDBFilePath string
}

func (o *surgeryBaseOptions) AddFlags(fs *pflag.FlagSet) {
	fs.StringVar(&o.outputDBFilePath, "output", o.outputDBFilePath, "path to the filePath db file")
	_ = cobra.MarkFlagRequired(fs, "output")
}

func (o *surgeryBaseOptions) Validate() error {
	if o.outputDBFilePath == "" {
		return errors.New("output database path wasn't given, specify output database file path with --output option")
	}
	return nil
}

func newSurgeryRevertMetaPageCommand() *cobra.Command {
	var o surgeryBaseOptions
	revertMetaPageCmd := &cobra.Command{
		Use:   "revert-meta-page <bbolt-file>",
		Short: "Revert the meta page to revert the changes performed by the latest transaction",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := o.Validate(); err != nil {
				return err
			}
			return surgeryRevertMetaPageFunc(args[0], o)
		},
	}
	o.AddFlags(revertMetaPageCmd.Flags())
	return revertMetaPageCmd
}

func surgeryRevertMetaPageFunc(srcDBPath string, cfg surgeryBaseOptions) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	if err := common.CopyFile(srcDBPath, cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("[revert-meta-page] copy file failed: %w", err)
	}

	if err := surgeon.RevertMetaPage(cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("revert-meta-page command failed: %w", err)
	}

	fmt.Fprintln(os.Stdout, "The meta page is reverted.")

	return nil
}

type surgeryCopyPageOptions struct {
	surgeryBaseOptions
	sourcePageId      uint64
	destinationPageId uint64
}

func (o *surgeryCopyPageOptions) AddFlags(fs *pflag.FlagSet) {
	o.surgeryBaseOptions.AddFlags(fs)
	fs.Uint64VarP(&o.sourcePageId, "from-page", "", o.sourcePageId, "source page Id")
	fs.Uint64VarP(&o.destinationPageId, "to-page", "", o.destinationPageId, "destination page Id")
	_ = cobra.MarkFlagRequired(fs, "from-page")
	_ = cobra.MarkFlagRequired(fs, "to-page")
}

func (o *surgeryCopyPageOptions) Validate() error {
	if err := o.surgeryBaseOptions.Validate(); err != nil {
		return err
	}
	if o.sourcePageId == o.destinationPageId {
		return fmt.Errorf("'--from-page' and '--to-page' have the same value: %d", o.sourcePageId)
	}
	return nil
}

func newSurgeryCopyPageCommand() *cobra.Command {
	var o surgeryCopyPageOptions
	copyPageCmd := &cobra.Command{
		Use:   "copy-page <bbolt-file>",
		Short: "Copy page from the source page Id to the destination page Id",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := o.Validate(); err != nil {
				return err
			}
			return surgeryCopyPageFunc(args[0], o)
		},
	}
	o.AddFlags(copyPageCmd.Flags())
	return copyPageCmd
}

func surgeryCopyPageFunc(srcDBPath string, cfg surgeryCopyPageOptions) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	if err := common.CopyFile(srcDBPath, cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("[copy-page] copy file failed: %w", err)
	}

	if err := surgeon.CopyPage(cfg.outputDBFilePath, common.Pgid(cfg.sourcePageId), common.Pgid(cfg.destinationPageId)); err != nil {
		return fmt.Errorf("copy-page command failed: %w", err)
	}

	meta, err := readMetaPage(srcDBPath)
	if err != nil {
		return err
	}
	if meta.IsFreelistPersisted() {
		fmt.Fprintf(os.Stdout, "WARNING: the free list might have changed.\n")
		fmt.Fprintf(os.Stdout, "Please consider executing `./bbolt surgery freelist abandon ...`\n")
	}

	fmt.Fprintf(os.Stdout, "The page %d was successfully copied to page %d\n", cfg.sourcePageId, cfg.destinationPageId)
	return nil
}

type surgeryClearPageOptions struct {
	surgeryBaseOptions
	pageId uint64
}

func (o *surgeryClearPageOptions) AddFlags(fs *pflag.FlagSet) {
	o.surgeryBaseOptions.AddFlags(fs)
	fs.Uint64VarP(&o.pageId, "pageId", "", o.pageId, "page Id")
	_ = cobra.MarkFlagRequired(fs, "pageId")
}

func (o *surgeryClearPageOptions) Validate() error {
	if err := o.surgeryBaseOptions.Validate(); err != nil {
		return err
	}
	if o.pageId < 2 {
		return fmt.Errorf("the pageId must be at least 2, but got %d", o.pageId)
	}
	return nil
}

func newSurgeryClearPageCommand() *cobra.Command {
	var o surgeryClearPageOptions
	clearPageCmd := &cobra.Command{
		Use:   "clear-page <bbolt-file>",
		Short: "Clears all elements from the given page, which can be a branch or leaf page",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := o.Validate(); err != nil {
				return err
			}
			return surgeryClearPageFunc(args[0], o)
		},
	}
	o.AddFlags(clearPageCmd.Flags())
	return clearPageCmd
}

func surgeryClearPageFunc(srcDBPath string, cfg surgeryClearPageOptions) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	if err := common.CopyFile(srcDBPath, cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("[clear-page] copy file failed: %w", err)
	}

	needAbandonFreelist, err := surgeon.ClearPage(cfg.outputDBFilePath, common.Pgid(cfg.pageId))
	if err != nil {
		return fmt.Errorf("clear-page command failed: %w", err)
	}

	if needAbandonFreelist {
		fmt.Fprintf(os.Stdout, "WARNING: The clearing has abandoned some pages that are not yet referenced from free list.\n")
		fmt.Fprintf(os.Stdout, "Please consider executing `./bbolt surgery freelist abandon ...`\n")
	}

	fmt.Fprintf(os.Stdout, "The page (%d) was cleared\n", cfg.pageId)
	return nil
}

type surgeryClearPageElementsOptions struct {
	surgeryBaseOptions
	pageId          uint64
	startElementIdx int
	endElementIdx   int
}

func (o *surgeryClearPageElementsOptions) AddFlags(fs *pflag.FlagSet) {
	o.surgeryBaseOptions.AddFlags(fs)
	fs.Uint64VarP(&o.pageId, "pageId", "", o.pageId, "page id")
	fs.IntVarP(&o.startElementIdx, "from-index", "", o.startElementIdx, "start element index (included) to clear, starting from 0")
	fs.IntVarP(&o.endElementIdx, "to-index", "", o.endElementIdx, "end element index (excluded) to clear, starting from 0, -1 means to the end of page")
	_ = cobra.MarkFlagRequired(fs, "pageId")
	_ = cobra.MarkFlagRequired(fs, "from-index")
	_ = cobra.MarkFlagRequired(fs, "to-index")
}

func (o *surgeryClearPageElementsOptions) Validate() error {
	if err := o.surgeryBaseOptions.Validate(); err != nil {
		return err
	}
	if o.pageId < 2 {
		return fmt.Errorf("the pageId must be at least 2, but got %d", o.pageId)
	}
	return nil
}

func newSurgeryClearPageElementsCommand() *cobra.Command {
	var o surgeryClearPageElementsOptions
	clearElementCmd := &cobra.Command{
		Use:   "clear-page-elements <bbolt-file>",
		Short: "Clears elements from the given page, which can be a branch or leaf page",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := o.Validate(); err != nil {
				return err
			}
			return surgeryClearPageElementFunc(args[0], o)
		},
	}
	o.AddFlags(clearElementCmd.Flags())
	return clearElementCmd
}

func surgeryClearPageElementFunc(srcDBPath string, cfg surgeryClearPageElementsOptions) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	if err := common.CopyFile(srcDBPath, cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("[clear-page-element] copy file failed: %w", err)
	}

	needAbandonFreelist, err := surgeon.ClearPageElements(cfg.outputDBFilePath, common.Pgid(cfg.pageId), cfg.startElementIdx, cfg.endElementIdx, false)
	if err != nil {
		return fmt.Errorf("clear-page-element command failed: %w", err)
	}

	if needAbandonFreelist {
		fmt.Fprintf(os.Stdout, "WARNING: The clearing has abandoned some pages that are not yet referenced from free list.\n")
		fmt.Fprintf(os.Stdout, "Please consider executing `./bbolt surgery freelist abandon ...`\n")
	}

	fmt.Fprintf(os.Stdout, "All elements in [%d, %d) in page %d were cleared\n", cfg.startElementIdx, cfg.endElementIdx, cfg.pageId)
	return nil
}

func readMetaPage(path string) (*common.Meta, error) {
	pageSize, _, err := guts_cli.ReadPageAndHWMSize(path)
	if err != nil {
		return nil, fmt.Errorf("read Page size failed: %w", err)
	}

	m := make([]*common.Meta, 2)
	for i := 0; i < 2; i++ {
		m[i], _, err = ReadMetaPageAt(path, uint32(i), uint32(pageSize))
		if err != nil {
			return nil, fmt.Errorf("read meta page %d failed: %w", i, err)
		}
	}

	if m[0].Txid() > m[1].Txid() {
		return m[0], nil
	}
	return m[1], nil
}

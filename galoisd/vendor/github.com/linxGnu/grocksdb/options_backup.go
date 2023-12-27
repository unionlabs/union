package grocksdb

// #include "rocksdb/c.h"
// #include "grocksdb.h"
import "C"
import "unsafe"

// ShareFilesNaming describes possible naming schemes for backup
// table file names when the table files are stored in the shared_checksum
// directory (i.e., both share_table_files and share_files_with_checksum
// are true).
type ShareFilesNaming uint32

const (
	// LegacyCrc32cAndFileSize indicates backup SST filenames are <file_number>_<crc32c>_<file_size>.sst
	// where <crc32c> is an unsigned decimal integer. This is the
	// original/legacy naming scheme for share_files_with_checksum,
	// with two problems:
	// * At massive scale, collisions on this triple with different file
	//   contents is plausible.
	// * Determining the name to use requires computing the checksum,
	//   so generally requires reading the whole file even if the file
	//   is already backed up.
	// ** ONLY RECOMMENDED FOR PRESERVING OLD BEHAVIOR **
	LegacyCrc32cAndFileSize ShareFilesNaming = 1

	// UseDBSessionID indicates backup SST filenames are <file_number>_s<db_session_id>.sst. This
	// pair of values should be very strongly unique for a given SST file
	// and easily determined before computing a checksum. The 's' indicates
	// the value is a DB session id, not a checksum.
	//
	// Exceptions:
	// * For old SST files without a DB session id, kLegacyCrc32cAndFileSize
	//   will be used instead, matching the names assigned by RocksDB versions
	//   not supporting the newer naming scheme.
	// * See also flags below.
	UseDBSessionID ShareFilesNaming = 2

	MaskNoNamingFlags ShareFilesNaming = 0xffff

	// FlagIncludeFileSize if not already part of the naming scheme, insert
	//   _<file_size>
	// before .sst in the name. In case of user code actually parsing the
	// last _<whatever> before the .sst as the file size, this preserves that
	// feature of kLegacyCrc32cAndFileSize. In other words, this option makes
	// official that unofficial feature of the backup metadata.
	//
	// We do not consider SST file sizes to have sufficient entropy to
	// contribute significantly to naming uniqueness.
	FlagIncludeFileSize ShareFilesNaming = 1 << 31

	// FlagMatchInterimNaming indicates when encountering an SST file from a Facebook-internal early
	// release of 6.12, use the default naming scheme in effect for
	// when the SST file was generated (assuming full file checksum
	// was not set to GetFileChecksumGenCrc32cFactory()). That naming is
	// <file_number>_<db_session_id>.sst
	// and ignores kFlagIncludeFileSize setting.
	// NOTE: This flag is intended to be temporary and should be removed
	// in a later release.
	FlagMatchInterimNaming ShareFilesNaming = 1 << 30

	MaskNamingFlags ShareFilesNaming = ^MaskNoNamingFlags
)

// BackupEngineOptions represents options for backup engine.
type BackupEngineOptions struct {
	c *C.rocksdb_backup_engine_options_t
}

// NewBackupableDBOptions
func NewBackupableDBOptions(backupDir string) *BackupEngineOptions {
	cDir := C.CString(backupDir)
	op := C.rocksdb_backup_engine_options_create(cDir)
	C.free(unsafe.Pointer(cDir))
	return &BackupEngineOptions{c: op}
}

// SetBackupDir sets where to keep the backup files. Has to be different than dbname_
// Best to set this to dbname_ + "/backups".
func (b *BackupEngineOptions) SetBackupDir(dir string) {
	cDir := C.CString(dir)
	C.rocksdb_backup_engine_options_set_backup_dir(b.c, cDir)
	C.free(unsafe.Pointer(cDir))
}

// SetEnv to be used for backup file I/O. If it's
// nullptr, backups will be written out using DBs Env. If it's
// non-nullptr, backup's I/O will be performed using this object.
// If you want to have backups on HDFS, use HDFS Env here!
func (b *BackupEngineOptions) SetEnv(env *Env) {
	C.rocksdb_backup_engine_options_set_env(b.c, env.c)
}

// ShareTableFiles if set to true, backup will assume that table files with
// same name have the same contents. This enables incremental backups and
// avoids unnecessary data copies.
//
// If false, each backup will be on its own and will
// not share any data with other backups.
//
// Default: true
func (b *BackupEngineOptions) ShareTableFiles(flag bool) {
	C.rocksdb_backup_engine_options_set_share_table_files(b.c, boolToChar(flag))
}

// IsShareTableFiles returns if backup will assume that table files with
// same name have the same contents. This enables incremental backups and
// avoids unnecessary data copies.
//
// If false, each backup will be on its own and will
// not share any data with other backups.
func (b *BackupEngineOptions) IsShareTableFiles() bool {
	return charToBool(C.rocksdb_backup_engine_options_get_share_table_files(b.c))
}

// SetSync if true, we can guarantee you'll get consistent backup even
// on a machine crash/reboot. Backup process is slower with sync enabled.
//
// If false, we don't guarantee anything on machine reboot. However,
// chances are some of the backups are consistent.
//
// Default: true
func (b *BackupEngineOptions) SetSync(flag bool) {
	C.rocksdb_backup_engine_options_set_sync(b.c, boolToChar(flag))
}

// IsSync if true, we can guarantee you'll get consistent backup even
// on a machine crash/reboot. Backup process is slower with sync enabled.
//
// If false, we don't guarantee anything on machine reboot. However,
// chances are some of the backups are consistent.
func (b *BackupEngineOptions) IsSync() bool {
	return charToBool(C.rocksdb_backup_engine_options_get_sync(b.c))
}

// DestroyOldData if true, it will delete whatever backups there are already
//
// Default: false
func (b *BackupEngineOptions) DestroyOldData(flag bool) {
	C.rocksdb_backup_engine_options_set_destroy_old_data(b.c, boolToChar(flag))
}

// IsDestroyOldData indicates if we should delete whatever backups there are already.
func (b *BackupEngineOptions) IsDestroyOldData() bool {
	return charToBool(C.rocksdb_backup_engine_options_get_destroy_old_data(b.c))
}

// BackupLogFiles if false, we won't backup log files. This option can be useful for backing
// up in-memory databases where log file are persisted, but table files are in
// memory.
//
// Default: true
func (b *BackupEngineOptions) BackupLogFiles(flag bool) {
	C.rocksdb_backup_engine_options_set_backup_log_files(b.c, boolToChar(flag))
}

// IsBackupLogFiles if false, we won't backup log files. This option can be useful for backing
// up in-memory databases where log file are persisted, but table files are in
// memory.
func (b *BackupEngineOptions) IsBackupLogFiles() bool {
	return charToBool(C.rocksdb_backup_engine_options_get_backup_log_files(b.c))
}

// SetBackupRateLimit sets max bytes that can be transferred in a second during backup.
// If 0, go as fast as you can.
//
// Default: 0
func (b *BackupEngineOptions) SetBackupRateLimit(limit uint64) {
	C.rocksdb_backup_engine_options_set_backup_rate_limit(b.c, C.uint64_t(limit))
}

// GetBackupRateLimit gets max bytes that can be transferred in a second during backup.
// If 0, go as fast as you can.
func (b *BackupEngineOptions) GetBackupRateLimit() uint64 {
	return uint64(C.rocksdb_backup_engine_options_get_backup_rate_limit(b.c))
}

// SetRestoreRateLimit sets max bytes that can be transferred in a second during restore.
// If 0, go as fast as you can
//
// Default: 0
func (b *BackupEngineOptions) SetRestoreRateLimit(limit uint64) {
	C.rocksdb_backup_engine_options_set_restore_rate_limit(b.c, C.uint64_t(limit))
}

// GetRestoreRateLimit gets max bytes that can be transferred in a second during restore.
// If 0, go as fast as you can
func (b *BackupEngineOptions) GetRestoreRateLimit() uint64 {
	return uint64(C.rocksdb_backup_engine_options_get_restore_rate_limit(b.c))
}

// SetMaxBackgroundOperations sets max number of background threads will copy files for CreateNewBackup()
// and RestoreDBFromBackup()
//
// Default: 1
func (b *BackupEngineOptions) SetMaxBackgroundOperations(v int) {
	C.rocksdb_backup_engine_options_set_max_background_operations(b.c, C.int(v))
}

// GetMaxBackgroundOperations gets max number of background threads will copy files for CreateNewBackup()
// and RestoreDBFromBackup()
func (b *BackupEngineOptions) GetMaxBackgroundOperations() int {
	return int(C.rocksdb_backup_engine_options_get_max_background_operations(b.c))
}

// SetCallbackTriggerIntervalSize sets size (N) during backup user can get callback every time next
// N bytes being copied.
//
// Default: N=4194304
func (b *BackupEngineOptions) SetCallbackTriggerIntervalSize(size uint64) {
	C.rocksdb_backup_engine_options_set_callback_trigger_interval_size(b.c, C.uint64_t(size))
}

// GetCallbackTriggerIntervalSize gets size (N) during backup user can get callback every time next
// N bytes being copied.
func (b *BackupEngineOptions) GetCallbackTriggerIntervalSize() uint64 {
	return uint64(C.rocksdb_backup_engine_options_get_callback_trigger_interval_size(b.c))
}

// SetMaxValidBackupsToOpen sets max number of valid backup to open.
//
// For BackupEngineReadOnly, Open() will open at most this many of the
// latest non-corrupted backups.
//
// Note: this setting is ignored (behaves like INT_MAX) for any kind of
// writable BackupEngine because it would inhibit accounting for shared
// files for proper backup deletion, including purging any incompletely
// created backups on creation of a new backup.
//
// Default: INT_MAX
func (b *BackupEngineOptions) SetMaxValidBackupsToOpen(val int) {
	C.rocksdb_backup_engine_options_set_max_valid_backups_to_open(b.c, C.int(val))
}

// GetMaxValidBackupsToOpen gets max number of valid backup to open.
//
// For BackupEngineReadOnly, Open() will open at most this many of the
// latest non-corrupted backups.
//
// Note: this setting is ignored (behaves like INT_MAX) for any kind of
// writable BackupEngine because it would inhibit accounting for shared
// files for proper backup deletion, including purging any incompletely
// created backups on creation of a new backup.
func (b *BackupEngineOptions) GetMaxValidBackupsToOpen() int {
	return int(C.rocksdb_backup_engine_options_get_max_valid_backups_to_open(b.c))
}

// SetShareFilesWithChecksumNaming sets naming option for share_files_with_checksum table files. See
// ShareFilesNaming for details.
//
// Modifying this option cannot introduce a downgrade compatibility issue
// because RocksDB can read, restore, and delete backups using different file
// names, and it's OK for a backup directory to use a mixture of table file
// naming schemes.
//
// However, modifying this option and saving more backups to the same
// directory can lead to the same file getting saved again to that
// directory, under the new shared name in addition to the old shared
// name.
//
// Default: UseDBSessionID | FlagIncludeFileSize | FlagMatchInterimNaming
func (b *BackupEngineOptions) SetShareFilesWithChecksumNaming(val ShareFilesNaming) {
	C.rocksdb_backup_engine_options_set_share_files_with_checksum_naming(b.c, C.int(val))
}

// GetShareFilesWithChecksumNaming gets naming option for share_files_with_checksum table files. See
// ShareFilesNaming for details.
func (b *BackupEngineOptions) GetShareFilesWithChecksumNaming() ShareFilesNaming {
	return ShareFilesNaming(C.rocksdb_backup_engine_options_get_share_files_with_checksum_naming(b.c))
}

// Destroy releases these options.
func (b *BackupEngineOptions) Destroy() {
	C.rocksdb_backup_engine_options_destroy(b.c)
}

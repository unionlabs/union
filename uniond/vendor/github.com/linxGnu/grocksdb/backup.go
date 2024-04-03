package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"
import (
	"unsafe"
)

// BackupInfo represents the information about a backup.
type BackupInfo struct {
	ID        uint32
	Timestamp int64
	Size      uint64
	NumFiles  uint32
}

// BackupEngine is a reusable handle to a RocksDB Backup, created by
// OpenBackupEngine.
type BackupEngine struct {
	c  *C.rocksdb_backup_engine_t
	db *DB
}

// OpenBackupEngine opens a backup engine with specified options.
func OpenBackupEngine(opts *Options, path string) (be *BackupEngine, err error) {
	cpath := C.CString(path)

	var cErr *C.char
	bEngine := C.rocksdb_backup_engine_open(opts.c, cpath, &cErr)
	if err = fromCError(cErr); err == nil {
		be = &BackupEngine{
			c: bEngine,
		}
	}

	C.free(unsafe.Pointer(cpath))
	return
}

// OpenBackupEngineWithOpt opens a backup engine with specified options.
func OpenBackupEngineWithOpt(opts *BackupEngineOptions, env *Env) (be *BackupEngine, err error) {
	var cErr *C.char
	bEngine := C.rocksdb_backup_engine_open_opts(opts.c, env.c, &cErr)
	if err = fromCError(cErr); err == nil {
		be = &BackupEngine{
			c: bEngine,
		}
	}

	return
}

// CreateBackupEngine opens a backup engine from DB.
func CreateBackupEngine(db *DB) (be *BackupEngine, err error) {
	if be, err = OpenBackupEngine(db.opts, db.Name()); err == nil {
		be.db = db
	}
	return
}

// CreateBackupEngineWithPath opens a backup engine from DB and path
func CreateBackupEngineWithPath(db *DB, path string) (be *BackupEngine, err error) {
	if be, err = OpenBackupEngine(db.opts, path); err == nil {
		be.db = db
	}
	return
}

// CreateNewBackup takes a new backup from db.
func (b *BackupEngine) CreateNewBackup() (err error) {
	var cErr *C.char
	C.rocksdb_backup_engine_create_new_backup(b.c, b.db.c, &cErr)
	err = fromCError(cErr)
	return
}

// CreateNewBackupFlush takes a new backup from db.
// Backup would be created after flushing.
func (b *BackupEngine) CreateNewBackupFlush(flushBeforeBackup bool) (err error) {
	var cErr *C.char
	C.rocksdb_backup_engine_create_new_backup_flush(b.c, b.db.c, boolToChar(flushBeforeBackup), &cErr)
	err = fromCError(cErr)
	return
}

// PurgeOldBackups deletes old backups, where `numBackupsToKeep` is how many backups you’d like to keep.
func (b *BackupEngine) PurgeOldBackups(numBackupsToKeep uint32) (err error) {
	var cErr *C.char
	C.rocksdb_backup_engine_purge_old_backups(b.c, C.uint32_t(numBackupsToKeep), &cErr)
	err = fromCError(cErr)
	return
}

// VerifyBackup verifies a backup by its id.
func (b *BackupEngine) VerifyBackup(backupID uint32) (err error) {
	var cErr *C.char
	C.rocksdb_backup_engine_verify_backup(b.c, C.uint32_t(backupID), &cErr)
	err = fromCError(cErr)
	return
}

// GetInfo gets an object that gives information about
// the backups that have already been taken
func (b *BackupEngine) GetInfo() (infos []BackupInfo) {
	info := C.rocksdb_backup_engine_get_backup_info(b.c)

	n := int(C.rocksdb_backup_engine_info_count(info))
	infos = make([]BackupInfo, n)
	for i := range infos {
		index := C.int(i)
		infos[i].ID = uint32(C.rocksdb_backup_engine_info_backup_id(info, index))
		infos[i].Timestamp = int64(C.rocksdb_backup_engine_info_timestamp(info, index))
		infos[i].Size = uint64(C.rocksdb_backup_engine_info_size(info, index))
		infos[i].NumFiles = uint32(C.rocksdb_backup_engine_info_number_files(info, index))
	}

	C.rocksdb_backup_engine_info_destroy(info)
	return
}

// RestoreDBFromLatestBackup restores the latest backup to dbDir. walDir
// is where the write ahead logs are restored to and usually the same as dbDir.
func (b *BackupEngine) RestoreDBFromLatestBackup(dbDir, walDir string, ro *RestoreOptions) (err error) {
	cDbDir := C.CString(dbDir)
	cWalDir := C.CString(walDir)

	var cErr *C.char
	C.rocksdb_backup_engine_restore_db_from_latest_backup(b.c, cDbDir, cWalDir, ro.c, &cErr)
	err = fromCError(cErr)

	C.free(unsafe.Pointer(cDbDir))
	C.free(unsafe.Pointer(cWalDir))
	return
}

// RestoreDBFromBackup restores the backup (identified by its id) to dbDir. walDir
// is where the write ahead logs are restored to and usually the same as dbDir.
func (b *BackupEngine) RestoreDBFromBackup(dbDir, walDir string, ro *RestoreOptions, backupID uint32) (err error) {
	cDbDir := C.CString(dbDir)
	cWalDir := C.CString(walDir)

	var cErr *C.char
	C.rocksdb_backup_engine_restore_db_from_backup(b.c, cDbDir, cWalDir, ro.c, C.uint32_t(backupID), &cErr)
	err = fromCError(cErr)

	C.free(unsafe.Pointer(cDbDir))
	C.free(unsafe.Pointer(cWalDir))
	return
}

// Close close the backup engine and cleans up state
// The backups already taken remain on storage.
func (b *BackupEngine) Close() {
	C.rocksdb_backup_engine_close(b.c)
	b.c = nil
	b.db = nil
}

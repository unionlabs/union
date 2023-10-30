#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_aggregate_context(param0: *mut sqlite3_context, nbytes: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_aggregate_count(param0: *mut sqlite3_context) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_auto_extension(xentrypoint: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_backup_finish(p: *mut sqlite3_backup) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_backup_init(pdest: *mut sqlite3, zdestname: ::windows_sys::core::PCSTR, psource: *mut sqlite3, zsourcename: ::windows_sys::core::PCSTR) -> *mut sqlite3_backup;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_backup_pagecount(p: *mut sqlite3_backup) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_backup_remaining(p: *mut sqlite3_backup) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_backup_step(p: *mut sqlite3_backup, npage: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_blob(param0: *mut sqlite3_stmt, param1: i32, param2: *const ::core::ffi::c_void, n: i32, param4: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_blob64(param0: *mut sqlite3_stmt, param1: i32, param2: *const ::core::ffi::c_void, param3: u64, param4: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_double(param0: *mut sqlite3_stmt, param1: i32, param2: f64) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_int(param0: *mut sqlite3_stmt, param1: i32, param2: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_int64(param0: *mut sqlite3_stmt, param1: i32, param2: i64) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_null(param0: *mut sqlite3_stmt, param1: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_parameter_count(param0: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_parameter_index(param0: *mut sqlite3_stmt, zname: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_parameter_name(param0: *mut sqlite3_stmt, param1: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_pointer(param0: *mut sqlite3_stmt, param1: i32, param2: *mut ::core::ffi::c_void, param3: ::windows_sys::core::PCSTR, param4: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_text(param0: *mut sqlite3_stmt, param1: i32, param2: ::windows_sys::core::PCSTR, param3: i32, param4: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_text16(param0: *mut sqlite3_stmt, param1: i32, param2: *const ::core::ffi::c_void, param3: i32, param4: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_text64(param0: *mut sqlite3_stmt, param1: i32, param2: ::windows_sys::core::PCSTR, param3: u64, param4: isize, encoding: u8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_value(param0: *mut sqlite3_stmt, param1: i32, param2: *const sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_zeroblob(param0: *mut sqlite3_stmt, param1: i32, n: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_bind_zeroblob64(param0: *mut sqlite3_stmt, param1: i32, param2: u64) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_blob_bytes(param0: *mut sqlite3_blob) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_blob_close(param0: *mut sqlite3_blob) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_blob_open(param0: *mut sqlite3, zdb: ::windows_sys::core::PCSTR, ztable: ::windows_sys::core::PCSTR, zcolumn: ::windows_sys::core::PCSTR, irow: i64, flags: i32, ppblob: *mut *mut sqlite3_blob) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_blob_read(param0: *mut sqlite3_blob, z: *mut ::core::ffi::c_void, n: i32, ioffset: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_blob_reopen(param0: *mut sqlite3_blob, param1: i64) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_blob_write(param0: *mut sqlite3_blob, z: *const ::core::ffi::c_void, n: i32, ioffset: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_busy_handler(param0: *mut sqlite3, param1: isize, param2: *mut ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_busy_timeout(param0: *mut sqlite3, ms: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_cancel_auto_extension(xentrypoint: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_changes(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_clear_bindings(param0: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_close(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_close_v2(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_collation_needed(param0: *mut sqlite3, param1: *mut ::core::ffi::c_void, param2: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_collation_needed16(param0: *mut sqlite3, param1: *mut ::core::ffi::c_void, param2: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_blob(param0: *mut sqlite3_stmt, icol: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_bytes(param0: *mut sqlite3_stmt, icol: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_bytes16(param0: *mut sqlite3_stmt, icol: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_count(pstmt: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_database_name(param0: *mut sqlite3_stmt, param1: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_database_name16(param0: *mut sqlite3_stmt, param1: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_decltype(param0: *mut sqlite3_stmt, param1: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_decltype16(param0: *mut sqlite3_stmt, param1: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_double(param0: *mut sqlite3_stmt, icol: i32) -> f64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_int(param0: *mut sqlite3_stmt, icol: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_int64(param0: *mut sqlite3_stmt, icol: i32) -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_name(param0: *mut sqlite3_stmt, n: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_name16(param0: *mut sqlite3_stmt, n: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_origin_name(param0: *mut sqlite3_stmt, param1: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_origin_name16(param0: *mut sqlite3_stmt, param1: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_table_name(param0: *mut sqlite3_stmt, param1: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_table_name16(param0: *mut sqlite3_stmt, param1: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_text(param0: *mut sqlite3_stmt, icol: i32) -> *mut u8;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_text16(param0: *mut sqlite3_stmt, icol: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_type(param0: *mut sqlite3_stmt, icol: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_column_value(param0: *mut sqlite3_stmt, icol: i32) -> *mut sqlite3_value;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_commit_hook(param0: *mut sqlite3, param1: isize, param2: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_compileoption_get(n: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_compileoption_used(zoptname: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_complete(sql: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_complete16(sql: *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_config(param0: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_context_db_handle(param0: *mut sqlite3_context) -> *mut sqlite3;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_collation(param0: *mut sqlite3, zname: ::windows_sys::core::PCSTR, etextrep: i32, parg: *mut ::core::ffi::c_void, xcompare: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_collation16(param0: *mut sqlite3, zname: *const ::core::ffi::c_void, etextrep: i32, parg: *mut ::core::ffi::c_void, xcompare: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_collation_v2(param0: *mut sqlite3, zname: ::windows_sys::core::PCSTR, etextrep: i32, parg: *mut ::core::ffi::c_void, xcompare: isize, xdestroy: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_filename(zdatabase: ::windows_sys::core::PCSTR, zjournal: ::windows_sys::core::PCSTR, zwal: ::windows_sys::core::PCSTR, nparam: i32, azparam: *const *const i8) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_function(db: *mut sqlite3, zfunctionname: ::windows_sys::core::PCSTR, narg: i32, etextrep: i32, papp: *mut ::core::ffi::c_void, xfunc: isize, xstep: isize, xfinal: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_function16(db: *mut sqlite3, zfunctionname: *const ::core::ffi::c_void, narg: i32, etextrep: i32, papp: *mut ::core::ffi::c_void, xfunc: isize, xstep: isize, xfinal: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_function_v2(db: *mut sqlite3, zfunctionname: ::windows_sys::core::PCSTR, narg: i32, etextrep: i32, papp: *mut ::core::ffi::c_void, xfunc: isize, xstep: isize, xfinal: isize, xdestroy: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_module(db: *mut sqlite3, zname: ::windows_sys::core::PCSTR, p: *const sqlite3_module, pclientdata: *mut ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_module_v2(db: *mut sqlite3, zname: ::windows_sys::core::PCSTR, p: *const sqlite3_module, pclientdata: *mut ::core::ffi::c_void, xdestroy: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_create_window_function(db: *mut sqlite3, zfunctionname: ::windows_sys::core::PCSTR, narg: i32, etextrep: i32, papp: *mut ::core::ffi::c_void, xstep: isize, xfinal: isize, xvalue: isize, xinverse: isize, xdestroy: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_data_count(pstmt: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_database_file_object(param0: ::windows_sys::core::PCSTR) -> *mut sqlite3_file;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_cacheflush(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_config(param0: *mut sqlite3, op: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_filename(db: *mut sqlite3, zdbname: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_handle(param0: *mut sqlite3_stmt) -> *mut sqlite3;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_mutex(param0: *mut sqlite3) -> *mut sqlite3_mutex;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_readonly(db: *mut sqlite3, zdbname: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_release_memory(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_db_status(param0: *mut sqlite3, op: i32, pcur: *mut i32, phiwtr: *mut i32, resetflg: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_declare_vtab(param0: *mut sqlite3, zsql: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_deserialize(db: *mut sqlite3, zschema: ::windows_sys::core::PCSTR, pdata: *mut u8, szdb: i64, szbuf: i64, mflags: u32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_drop_modules(db: *mut sqlite3, azkeep: *const *const i8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_enable_load_extension(db: *mut sqlite3, onoff: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_enable_shared_cache(param0: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_errcode(db: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_errmsg(param0: *mut sqlite3) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_errmsg16(param0: *mut sqlite3) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_errstr(param0: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_exec(param0: *mut sqlite3, sql: ::windows_sys::core::PCSTR, callback: isize, param3: *mut ::core::ffi::c_void, errmsg: *mut *mut i8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_expanded_sql(pstmt: *mut sqlite3_stmt) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_expired(param0: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_extended_errcode(db: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_extended_result_codes(param0: *mut sqlite3, onoff: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_file_control(param0: *mut sqlite3, zdbname: ::windows_sys::core::PCSTR, op: i32, param3: *mut ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_filename_database(param0: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_filename_journal(param0: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_filename_wal(param0: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_finalize(pstmt: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_free(param0: *mut ::core::ffi::c_void);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_free_filename(param0: ::windows_sys::core::PCSTR);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_free_table(result: *mut *mut i8);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_get_autocommit(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_get_auxdata(param0: *mut sqlite3_context, n: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_get_table(db: *mut sqlite3, zsql: ::windows_sys::core::PCSTR, pazresult: *mut *mut *mut i8, pnrow: *mut i32, pncolumn: *mut i32, pzerrmsg: *mut *mut i8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_global_recover() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_hard_heap_limit64(n: i64) -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_initialize() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_interrupt(param0: *mut sqlite3);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_keyword_check(param0: ::windows_sys::core::PCSTR, param1: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_keyword_count() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_keyword_name(param0: i32, param1: *const *const i8, param2: *mut i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_last_insert_rowid(param0: *mut sqlite3) -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_libversion() -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_libversion_number() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_limit(param0: *mut sqlite3, id: i32, newval: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_load_extension(db: *mut sqlite3, zfile: ::windows_sys::core::PCSTR, zproc: ::windows_sys::core::PCSTR, pzerrmsg: *mut *mut i8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_log(ierrcode: i32, zformat: ::windows_sys::core::PCSTR);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_malloc(param0: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_malloc64(param0: u64) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_memory_alarm(param0: isize, param1: *mut ::core::ffi::c_void, param2: i64) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_memory_highwater(resetflag: i32) -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_memory_used() -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_mprintf(param0: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_msize(param0: *mut ::core::ffi::c_void) -> u64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_mutex_alloc(param0: i32) -> *mut sqlite3_mutex;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_mutex_enter(param0: *mut sqlite3_mutex);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_mutex_free(param0: *mut sqlite3_mutex);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_mutex_leave(param0: *mut sqlite3_mutex);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_mutex_try(param0: *mut sqlite3_mutex) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_next_stmt(pdb: *mut sqlite3, pstmt: *mut sqlite3_stmt) -> *mut sqlite3_stmt;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_open(filename: ::windows_sys::core::PCSTR, ppdb: *mut *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_open16(filename: *const ::core::ffi::c_void, ppdb: *mut *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_open_v2(filename: ::windows_sys::core::PCSTR, ppdb: *mut *mut sqlite3, flags: i32, zvfs: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_os_end() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_os_init() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_overload_function(param0: *mut sqlite3, zfuncname: ::windows_sys::core::PCSTR, narg: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_prepare(db: *mut sqlite3, zsql: ::windows_sys::core::PCSTR, nbyte: i32, ppstmt: *mut *mut sqlite3_stmt, pztail: *const *const i8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_prepare16(db: *mut sqlite3, zsql: *const ::core::ffi::c_void, nbyte: i32, ppstmt: *mut *mut sqlite3_stmt, pztail: *const *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_prepare16_v2(db: *mut sqlite3, zsql: *const ::core::ffi::c_void, nbyte: i32, ppstmt: *mut *mut sqlite3_stmt, pztail: *const *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_prepare16_v3(db: *mut sqlite3, zsql: *const ::core::ffi::c_void, nbyte: i32, prepflags: u32, ppstmt: *mut *mut sqlite3_stmt, pztail: *const *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_prepare_v2(db: *mut sqlite3, zsql: ::windows_sys::core::PCSTR, nbyte: i32, ppstmt: *mut *mut sqlite3_stmt, pztail: *const *const i8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_prepare_v3(db: *mut sqlite3, zsql: ::windows_sys::core::PCSTR, nbyte: i32, prepflags: u32, ppstmt: *mut *mut sqlite3_stmt, pztail: *const *const i8) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_profile(param0: *mut sqlite3, xprofile: isize, param2: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_progress_handler(param0: *mut sqlite3, param1: i32, param2: isize, param3: *mut ::core::ffi::c_void);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_randomness(n: i32, p: *mut ::core::ffi::c_void);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_realloc(param0: *mut ::core::ffi::c_void, param1: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_realloc64(param0: *mut ::core::ffi::c_void, param1: u64) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_release_memory(param0: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_reset(pstmt: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_reset_auto_extension();
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_blob(param0: *mut sqlite3_context, param1: *const ::core::ffi::c_void, param2: i32, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_blob64(param0: *mut sqlite3_context, param1: *const ::core::ffi::c_void, param2: u64, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_double(param0: *mut sqlite3_context, param1: f64);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_error(param0: *mut sqlite3_context, param1: ::windows_sys::core::PCSTR, param2: i32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_error16(param0: *mut sqlite3_context, param1: *const ::core::ffi::c_void, param2: i32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_error_code(param0: *mut sqlite3_context, param1: i32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_error_nomem(param0: *mut sqlite3_context);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_error_toobig(param0: *mut sqlite3_context);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_int(param0: *mut sqlite3_context, param1: i32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_int64(param0: *mut sqlite3_context, param1: i64);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_null(param0: *mut sqlite3_context);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_pointer(param0: *mut sqlite3_context, param1: *mut ::core::ffi::c_void, param2: ::windows_sys::core::PCSTR, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_subtype(param0: *mut sqlite3_context, param1: u32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_text(param0: *mut sqlite3_context, param1: ::windows_sys::core::PCSTR, param2: i32, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_text16(param0: *mut sqlite3_context, param1: *const ::core::ffi::c_void, param2: i32, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_text16be(param0: *mut sqlite3_context, param1: *const ::core::ffi::c_void, param2: i32, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_text16le(param0: *mut sqlite3_context, param1: *const ::core::ffi::c_void, param2: i32, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_text64(param0: *mut sqlite3_context, param1: ::windows_sys::core::PCSTR, param2: u64, param3: isize, encoding: u8);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_value(param0: *mut sqlite3_context, param1: *mut sqlite3_value);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_zeroblob(param0: *mut sqlite3_context, n: i32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_result_zeroblob64(param0: *mut sqlite3_context, n: u64) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_rollback_hook(param0: *mut sqlite3, param1: isize, param2: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_rtree_geometry_callback(db: *mut sqlite3, zgeom: ::windows_sys::core::PCSTR, xgeom: isize, pcontext: *mut ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_rtree_query_callback(db: *mut sqlite3, zqueryfunc: ::windows_sys::core::PCSTR, xqueryfunc: isize, pcontext: *mut ::core::ffi::c_void, xdestructor: isize) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_serialize(db: *mut sqlite3, zschema: ::windows_sys::core::PCSTR, pisize: *mut i64, mflags: u32) -> *mut u8;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_set_authorizer(param0: *mut sqlite3, xauth: isize, puserdata: *mut ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_set_auxdata(param0: *mut sqlite3_context, n: i32, param2: *mut ::core::ffi::c_void, param3: isize);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_set_last_insert_rowid(param0: *mut sqlite3, param1: i64);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_shutdown() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_sleep(param0: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_snprintf(param0: i32, param1: ::windows_sys::core::PCSTR, param2: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_soft_heap_limit(n: i32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_soft_heap_limit64(n: i64) -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_sourceid() -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_sql(pstmt: *mut sqlite3_stmt) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_status(op: i32, pcurrent: *mut i32, phighwater: *mut i32, resetflag: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_status64(op: i32, pcurrent: *mut i64, phighwater: *mut i64, resetflag: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_step(param0: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_stmt_busy(param0: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_stmt_isexplain(pstmt: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_stmt_readonly(pstmt: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_stmt_status(param0: *mut sqlite3_stmt, op: i32, resetflg: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_append(param0: *mut sqlite3_str, zin: ::windows_sys::core::PCSTR, n: i32);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_appendall(param0: *mut sqlite3_str, zin: ::windows_sys::core::PCSTR);
    #[doc = "*Required features: 'Win32_System_SqlLite', 'Win32_Foundation'*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn sqlite3_str_appendchar(param0: *mut sqlite3_str, n: i32, c: super::super::Foundation::CHAR);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_appendf(param0: *mut sqlite3_str, zformat: ::windows_sys::core::PCSTR);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_errcode(param0: *mut sqlite3_str) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_finish(param0: *mut sqlite3_str) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_length(param0: *mut sqlite3_str) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_new(param0: *mut sqlite3) -> *mut sqlite3_str;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_reset(param0: *mut sqlite3_str);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_value(param0: *mut sqlite3_str) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_str_vappendf(param0: *mut sqlite3_str, zformat: ::windows_sys::core::PCSTR, param2: *mut i8);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_strglob(zglob: ::windows_sys::core::PCSTR, zstr: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_stricmp(param0: ::windows_sys::core::PCSTR, param1: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_strlike(zglob: ::windows_sys::core::PCSTR, zstr: ::windows_sys::core::PCSTR, cesc: u32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_strnicmp(param0: ::windows_sys::core::PCSTR, param1: ::windows_sys::core::PCSTR, param2: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_system_errno(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_table_column_metadata(db: *mut sqlite3, zdbname: ::windows_sys::core::PCSTR, ztablename: ::windows_sys::core::PCSTR, zcolumnname: ::windows_sys::core::PCSTR, pzdatatype: *const *const i8, pzcollseq: *const *const i8, pnotnull: *mut i32, pprimarykey: *mut i32, pautoinc: *mut i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_test_control(op: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_thread_cleanup();
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_threadsafe() -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_total_changes(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_trace(param0: *mut sqlite3, xtrace: isize, param2: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_trace_v2(param0: *mut sqlite3, umask: u32, xcallback: isize, pctx: *mut ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_transfer_bindings(param0: *mut sqlite3_stmt, param1: *mut sqlite3_stmt) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_txn_state(param0: *mut sqlite3, zschema: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_update_hook(param0: *mut sqlite3, param1: isize, param2: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_uri_boolean(zfile: ::windows_sys::core::PCSTR, zparam: ::windows_sys::core::PCSTR, bdefault: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_uri_int64(param0: ::windows_sys::core::PCSTR, param1: ::windows_sys::core::PCSTR, param2: i64) -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_uri_key(zfilename: ::windows_sys::core::PCSTR, n: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_uri_parameter(zfilename: ::windows_sys::core::PCSTR, zparam: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_user_data(param0: *mut sqlite3_context) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_blob(param0: *mut sqlite3_value) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_bytes(param0: *mut sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_bytes16(param0: *mut sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_double(param0: *mut sqlite3_value) -> f64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_dup(param0: *const sqlite3_value) -> *mut sqlite3_value;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_free(param0: *mut sqlite3_value);
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_frombind(param0: *mut sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_int(param0: *mut sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_int64(param0: *mut sqlite3_value) -> i64;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_nochange(param0: *mut sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_numeric_type(param0: *mut sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_pointer(param0: *mut sqlite3_value, param1: ::windows_sys::core::PCSTR) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_subtype(param0: *mut sqlite3_value) -> u32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_text(param0: *mut sqlite3_value) -> *mut u8;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_text16(param0: *mut sqlite3_value) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_text16be(param0: *mut sqlite3_value) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_text16le(param0: *mut sqlite3_value) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_value_type(param0: *mut sqlite3_value) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vfs_find(zvfsname: ::windows_sys::core::PCSTR) -> *mut sqlite3_vfs;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vfs_register(param0: *mut sqlite3_vfs, makedflt: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vfs_unregister(param0: *mut sqlite3_vfs) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vmprintf(param0: ::windows_sys::core::PCSTR, param1: *mut i8) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vsnprintf(param0: i32, param1: ::windows_sys::core::PCSTR, param2: ::windows_sys::core::PCSTR, param3: *mut i8) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vtab_collation(param0: *mut sqlite3_index_info, param1: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vtab_config(param0: *mut sqlite3, op: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vtab_nochange(param0: *mut sqlite3_context) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_vtab_on_conflict(param0: *mut sqlite3) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_wal_autocheckpoint(db: *mut sqlite3, n: i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_wal_checkpoint(db: *mut sqlite3, zdb: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_wal_checkpoint_v2(db: *mut sqlite3, zdb: ::windows_sys::core::PCSTR, emode: i32, pnlog: *mut i32, pnckpt: *mut i32) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_wal_hook(param0: *mut sqlite3, param1: isize, param2: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_win32_set_directory(r#type: u32, zvalue: *mut ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_win32_set_directory16(r#type: u32, zvalue: *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: 'Win32_System_SqlLite'*"]
    pub fn sqlite3_win32_set_directory8(r#type: u32, zvalue: ::windows_sys::core::PCSTR) -> i32;
}
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const FTS5_TOKENIZE_AUX: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const FTS5_TOKENIZE_DOCUMENT: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const FTS5_TOKENIZE_PREFIX: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const FTS5_TOKENIZE_QUERY: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const FTS5_TOKEN_COLOCATED: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const FULLY_WITHIN: u32 = 2u32;
#[repr(C)]
pub struct Fts5Context(pub u8);
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct Fts5ExtensionApi {
    pub iVersion: i32,
    pub xUserData: isize,
    pub xColumnCount: isize,
    pub xRowCount: isize,
    pub xColumnTotalSize: isize,
    pub xTokenize: isize,
    pub xPhraseCount: isize,
    pub xPhraseSize: isize,
    pub xInstCount: isize,
    pub xInst: isize,
    pub xRowid: isize,
    pub xColumnText: isize,
    pub xColumnSize: isize,
    pub xQueryPhrase: isize,
    pub xSetAuxdata: isize,
    pub xGetAuxdata: isize,
    pub xPhraseFirst: isize,
    pub xPhraseNext: isize,
    pub xPhraseFirstColumn: isize,
    pub xPhraseNextColumn: isize,
}
impl ::core::marker::Copy for Fts5ExtensionApi {}
impl ::core::clone::Clone for Fts5ExtensionApi {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct Fts5PhraseIter {
    pub a: *const u8,
    pub b: *const u8,
}
impl ::core::marker::Copy for Fts5PhraseIter {}
impl ::core::clone::Clone for Fts5PhraseIter {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct Fts5Tokenizer(pub u8);
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const NOT_WITHIN: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const PARTLY_WITHIN: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE3_TEXT: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ABORT: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ACCESS_EXISTS: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ACCESS_READ: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ACCESS_READWRITE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ALTER_TABLE: u32 = 26u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ANALYZE: u32 = 28u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ANY: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ATTACH: u32 = 24u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_AUTH: u32 = 23u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_BLOB: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_BUSY: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CANTOPEN: u32 = 14u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESETAPPLY_INVERT: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESETAPPLY_NOSAVEPOINT: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESETSTART_INVERT: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_ABORT: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_CONFLICT: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_CONSTRAINT: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_DATA: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_FOREIGN_KEY: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_NOTFOUND: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_OMIT: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHANGESET_REPLACE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHECKPOINT_FULL: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHECKPOINT_PASSIVE: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHECKPOINT_RESTART: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CHECKPOINT_TRUNCATE: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_COVERING_INDEX_SCAN: u32 = 20u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_GETMALLOC: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_GETMUTEX: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_GETPCACHE: u32 = 15u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_GETPCACHE2: u32 = 19u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_HEAP: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_LOG: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_LOOKASIDE: u32 = 13u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_MALLOC: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_MEMDB_MAXSIZE: u32 = 29u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_MEMSTATUS: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_MMAP_SIZE: u32 = 22u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_MULTITHREAD: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_MUTEX: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_PAGECACHE: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_PCACHE: u32 = 14u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_PCACHE2: u32 = 18u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_PCACHE_HDRSZ: u32 = 24u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_PMASZ: u32 = 25u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_SCRATCH: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_SERIALIZED: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_SINGLETHREAD: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_SMALL_MALLOC: u32 = 27u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_SORTERREF_SIZE: u32 = 28u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_SQLLOG: u32 = 21u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_STMTJRNL_SPILL: u32 = 26u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_URI: u32 = 17u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONFIG_WIN32_HEAPSIZE: u32 = 23u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CONSTRAINT: u32 = 19u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_COPY: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CORRUPT: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_INDEX: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_TABLE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_TEMP_INDEX: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_TEMP_TABLE: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_TEMP_TRIGGER: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_TEMP_VIEW: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_TRIGGER: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_VIEW: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_CREATE_VTABLE: u32 = 29u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_DEFENSIVE: u32 = 1010u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_DQS_DDL: u32 = 1014u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_DQS_DML: u32 = 1013u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_ENABLE_FKEY: u32 = 1002u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_ENABLE_FTS3_TOKENIZER: u32 = 1004u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION: u32 = 1005u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_ENABLE_QPSG: u32 = 1007u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_ENABLE_TRIGGER: u32 = 1003u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_ENABLE_VIEW: u32 = 1015u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_LEGACY_ALTER_TABLE: u32 = 1012u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_LEGACY_FILE_FORMAT: u32 = 1016u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_LOOKASIDE: u32 = 1001u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_MAINDBNAME: u32 = 1000u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_MAX: u32 = 1017u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE: u32 = 1006u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_RESET_DATABASE: u32 = 1009u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_TRIGGER_EQP: u32 = 1008u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_TRUSTED_SCHEMA: u32 = 1017u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBCONFIG_WRITABLE_SCHEMA: u32 = 1011u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_CACHE_HIT: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_CACHE_MISS: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_CACHE_SPILL: u32 = 12u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_CACHE_USED: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_CACHE_USED_SHARED: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_CACHE_WRITE: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_DEFERRED_FKS: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_LOOKASIDE_HIT: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_LOOKASIDE_MISS_FULL: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_LOOKASIDE_MISS_SIZE: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_LOOKASIDE_USED: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_MAX: u32 = 12u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_SCHEMA_USED: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DBSTATUS_STMT_USED: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DELETE: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DENY: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DESERIALIZE_FREEONCLOSE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DESERIALIZE_READONLY: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DESERIALIZE_RESIZEABLE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DETACH: u32 = 25u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DETERMINISTIC: u64 = 2048u64;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DIRECTONLY: u64 = 524288u64;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DONE: u32 = 101u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_INDEX: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_TABLE: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_TEMP_INDEX: u32 = 12u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_TEMP_TABLE: u32 = 13u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_TEMP_TRIGGER: u32 = 14u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_TEMP_VIEW: u32 = 15u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_TRIGGER: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_VIEW: u32 = 17u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_DROP_VTABLE: u32 = 30u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_EMPTY: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ERROR: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FAIL: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_BEGIN_ATOMIC_WRITE: u32 = 31u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_BUSYHANDLER: u32 = 15u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_CHUNK_SIZE: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_CKPT_DONE: u32 = 37u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_CKPT_START: u32 = 39u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_COMMIT_ATOMIC_WRITE: u32 = 32u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_COMMIT_PHASETWO: u32 = 22u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_DATA_VERSION: u32 = 35u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_FILE_POINTER: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_GET_LOCKPROXYFILE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_HAS_MOVED: u32 = 20u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_JOURNAL_POINTER: u32 = 28u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_LAST_ERRNO: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_LOCKSTATE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_LOCK_TIMEOUT: u32 = 34u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_MMAP_SIZE: u32 = 18u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_OVERWRITE: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_PDB: u32 = 30u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_PERSIST_WAL: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_POWERSAFE_OVERWRITE: u32 = 13u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_PRAGMA: u32 = 14u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_RBU: u32 = 26u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_RESERVE_BYTES: u32 = 38u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_ROLLBACK_ATOMIC_WRITE: u32 = 33u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_SET_LOCKPROXYFILE: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_SIZE_HINT: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_SIZE_LIMIT: u32 = 36u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_SYNC: u32 = 21u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_SYNC_OMITTED: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_TEMPFILENAME: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_TRACE: u32 = 19u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_VFSNAME: u32 = 12u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_VFS_POINTER: u32 = 27u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_WAL_BLOCK: u32 = 24u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_WIN32_AV_RETRY: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_WIN32_GET_HANDLE: u32 = 29u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_WIN32_SET_HANDLE: u32 = 23u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FCNTL_ZIPVFS: u32 = 25u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FLOAT: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FORMAT: u32 = 24u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FULL: u32 = 13u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_FUNCTION: u32 = 31u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_GET_LOCKPROXYFILE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IGNORE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_EQ: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_FUNCTION: u32 = 150u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_GE: u32 = 32u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_GLOB: u32 = 66u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_GT: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_IS: u32 = 72u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_ISNOT: u32 = 69u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_ISNOTNULL: u32 = 70u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_ISNULL: u32 = 71u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_LE: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_LIKE: u32 = 65u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_LT: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_MATCH: u32 = 64u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_NE: u32 = 68u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_CONSTRAINT_REGEXP: u32 = 67u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INDEX_SCAN_UNIQUE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INNOCUOUS: u64 = 2097152u64;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INSERT: u32 = 18u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INTEGER: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INTERNAL: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_INTERRUPT: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC16K: u32 = 64u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC1K: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC2K: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC32K: u32 = 128u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC4K: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC512: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC64K: u32 = 256u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_ATOMIC8K: u32 = 32u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_BATCH_ATOMIC: u32 = 16384u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_IMMUTABLE: u32 = 8192u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_POWERSAFE_OVERWRITE: u32 = 4096u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_SAFE_APPEND: u32 = 512u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_SEQUENTIAL: u32 = 1024u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOCAP_UNDELETABLE_WHEN_OPEN: u32 = 2048u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_IOERR: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LAST_ERRNO: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_ATTACHED: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_COLUMN: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_COMPOUND_SELECT: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_EXPR_DEPTH: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_FUNCTION_ARG: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_LENGTH: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_LIKE_PATTERN_LENGTH: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_SQL_LENGTH: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_TRIGGER_DEPTH: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_VARIABLE_NUMBER: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_VDBE_OP: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LIMIT_WORKER_THREADS: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LOCKED: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LOCK_EXCLUSIVE: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LOCK_NONE: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LOCK_PENDING: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LOCK_RESERVED: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_LOCK_SHARED: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MISMATCH: u32 = 20u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MISUSE: u32 = 21u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_FAST: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_RECURSIVE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_APP1: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_APP2: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_APP3: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_LRU: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_LRU2: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_MAIN: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_MASTER: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_MEM: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_MEM2: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_OPEN: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_PMEM: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_PRNG: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_VFS1: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_VFS2: u32 = 12u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_MUTEX_STATIC_VFS3: u32 = 13u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_NOLFS: u32 = 22u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_NOMEM: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_NOTADB: u32 = 26u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_NOTFOUND: u32 = 12u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_NOTICE: u32 = 27u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_NULL: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OK: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_AUTOPROXY: u32 = 32u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_CREATE: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_DELETEONCLOSE: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_EXCLUSIVE: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_FULLMUTEX: u32 = 65536u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_MAIN_DB: u32 = 256u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_MAIN_JOURNAL: u32 = 2048u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_MASTER_JOURNAL: u32 = 16384u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_MEMORY: u32 = 128u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_NOFOLLOW: u32 = 16777216u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_NOMUTEX: u32 = 32768u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_PRIVATECACHE: u32 = 262144u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_READONLY: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_READWRITE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_SHAREDCACHE: u32 = 131072u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_SUBJOURNAL: u32 = 8192u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_SUPER_JOURNAL: u32 = 16384u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_TEMP_DB: u32 = 512u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_TEMP_JOURNAL: u32 = 4096u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_TRANSIENT_DB: u32 = 1024u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_URI: u32 = 64u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_OPEN_WAL: u32 = 524288u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_PERM: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_PRAGMA: u32 = 19u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_PREPARE_NORMALIZE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_PREPARE_NO_VTAB: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_PREPARE_PERSISTENT: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_PROTOCOL: u32 = 15u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_RANGE: u32 = 25u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_READ: u32 = 20u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_READONLY: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_RECURSIVE: u32 = 33u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_REINDEX: u32 = 27u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_REPLACE: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ROLLBACK: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_ROW: u32 = 100u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SAVEPOINT: u32 = 32u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SCANSTAT_EST: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SCANSTAT_EXPLAIN: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SCANSTAT_NAME: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SCANSTAT_NLOOP: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SCANSTAT_NVISIT: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SCANSTAT_SELECTID: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SCHEMA: u32 = 17u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SELECT: u32 = 21u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SERIALIZE_NOCOPY: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SESSION_CONFIG_STRMSIZE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SET_LOCKPROXYFILE: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SHM_EXCLUSIVE: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SHM_LOCK: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SHM_NLOCK: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SHM_SHARED: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SHM_UNLOCK: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SOURCE_ID: &'static str = "2019-07-10 17:32:03 fc82b73eaac8b36950e527f12c4b5dc1e147e6f4ad2217ae43ad82882a88bfa6";
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_MALLOC_COUNT: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_MALLOC_SIZE: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_MEMORY_USED: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_PAGECACHE_OVERFLOW: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_PAGECACHE_SIZE: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_PAGECACHE_USED: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_PARSER_STACK: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_SCRATCH_OVERFLOW: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_SCRATCH_SIZE: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STATUS_SCRATCH_USED: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STMTSTATUS_AUTOINDEX: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STMTSTATUS_FULLSCAN_STEP: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STMTSTATUS_MEMUSED: u32 = 99u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STMTSTATUS_REPREPARE: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STMTSTATUS_RUN: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STMTSTATUS_SORT: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_STMTSTATUS_VM_STEP: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SUBTYPE: u64 = 1048576u64;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SYNC_DATAONLY: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SYNC_FULL: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_SYNC_NORMAL: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_ALWAYS: u32 = 13u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_ASSERT: u32 = 12u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_BENIGN_MALLOC_HOOKS: u32 = 10u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_BITVEC_TEST: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_BYTEORDER: u32 = 22u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_EXPLAIN_STMT: u32 = 19u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_EXTRA_SCHEMA_CHECKS: u32 = 29u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_FAULT_INSTALL: u32 = 9u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_FIRST: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_IMPOSTER: u32 = 25u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_INTERNAL_FUNCTIONS: u32 = 17u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_ISINIT: u32 = 23u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_ISKEYWORD: u32 = 16u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_LAST: u32 = 30u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_LOCALTIME_FAULT: u32 = 18u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_NEVER_CORRUPT: u32 = 20u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_ONCE_RESET_THRESHOLD: u32 = 19u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_OPTIMIZATIONS: u32 = 15u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_PARSER_COVERAGE: u32 = 26u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_PENDING_BYTE: u32 = 11u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_PRNG_RESET: u32 = 7u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_PRNG_RESTORE: u32 = 6u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_PRNG_SAVE: u32 = 5u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_PRNG_SEED: u32 = 28u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_RESERVE: u32 = 14u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_RESULT_INTREAL: u32 = 27u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_SCRATCHMALLOC: u32 = 17u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_SEEK_COUNT: u32 = 30u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_SORTER_MMAP: u32 = 24u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TESTCTRL_VDBE_COVERAGE: u32 = 21u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TEXT: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TOOBIG: u32 = 18u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TRACE_CLOSE: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TRACE_PROFILE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TRACE_ROW: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TRACE_STMT: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TRANSACTION: u32 = 22u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TXN_NONE: u32 = 0u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TXN_READ: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_TXN_WRITE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_UPDATE: u32 = 23u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_UTF16: u32 = 4u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_UTF16BE: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_UTF16LE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_UTF16_ALIGNED: u32 = 8u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_UTF8: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_VERSION: &'static str = "3.29.0";
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_VERSION_NUMBER: u32 = 3029000u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_VTAB_CONSTRAINT_SUPPORT: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_VTAB_DIRECTONLY: u32 = 3u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_VTAB_INNOCUOUS: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_WARNING: u32 = 28u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_WIN32_DATA_DIRECTORY_TYPE: u32 = 1u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const SQLITE_WIN32_TEMP_DIRECTORY_TYPE: u32 = 2u32;
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub const __SQLITESESSION_H_: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct fts5_api {
    pub iVersion: i32,
    pub xCreateTokenizer: isize,
    pub xFindTokenizer: isize,
    pub xCreateFunction: isize,
}
impl ::core::marker::Copy for fts5_api {}
impl ::core::clone::Clone for fts5_api {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub type fts5_extension_function = ::core::option::Option<unsafe extern "system" fn(papi: *const Fts5ExtensionApi, pfts: *mut Fts5Context, pctx: *mut sqlite3_context, nval: i32, apval: *mut *mut sqlite3_value)>;
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct fts5_tokenizer {
    pub xCreate: isize,
    pub xDelete: isize,
    pub xTokenize: isize,
}
impl ::core::marker::Copy for fts5_tokenizer {}
impl ::core::clone::Clone for fts5_tokenizer {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct sqlite3(pub u8);
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct sqlite3_api_routines {
    pub aggregate_context: isize,
    pub aggregate_count: isize,
    pub bind_blob: isize,
    pub bind_double: isize,
    pub bind_int: isize,
    pub bind_int64: isize,
    pub bind_null: isize,
    pub bind_parameter_count: isize,
    pub bind_parameter_index: isize,
    pub bind_parameter_name: isize,
    pub bind_text: isize,
    pub bind_text16: isize,
    pub bind_value: isize,
    pub busy_handler: isize,
    pub busy_timeout: isize,
    pub changes: isize,
    pub close: isize,
    pub collation_needed: isize,
    pub collation_needed16: isize,
    pub column_blob: isize,
    pub column_bytes: isize,
    pub column_bytes16: isize,
    pub column_count: isize,
    pub column_database_name: isize,
    pub column_database_name16: isize,
    pub column_decltype: isize,
    pub column_decltype16: isize,
    pub column_double: isize,
    pub column_int: isize,
    pub column_int64: isize,
    pub column_name: isize,
    pub column_name16: isize,
    pub column_origin_name: isize,
    pub column_origin_name16: isize,
    pub column_table_name: isize,
    pub column_table_name16: isize,
    pub column_text: isize,
    pub column_text16: isize,
    pub column_type: isize,
    pub column_value: isize,
    pub commit_hook: isize,
    pub complete: isize,
    pub complete16: isize,
    pub create_collation: isize,
    pub create_collation16: isize,
    pub create_function: isize,
    pub create_function16: isize,
    pub create_module: isize,
    pub data_count: isize,
    pub db_handle: isize,
    pub declare_vtab: isize,
    pub enable_shared_cache: isize,
    pub errcode: isize,
    pub errmsg: isize,
    pub errmsg16: isize,
    pub exec: isize,
    pub expired: isize,
    pub finalize: isize,
    pub free: isize,
    pub free_table: isize,
    pub get_autocommit: isize,
    pub get_auxdata: isize,
    pub get_table: isize,
    pub global_recover: isize,
    pub interruptx: isize,
    pub last_insert_rowid: isize,
    pub libversion: isize,
    pub libversion_number: isize,
    pub malloc: isize,
    pub mprintf: isize,
    pub open: isize,
    pub open16: isize,
    pub prepare: isize,
    pub prepare16: isize,
    pub profile: isize,
    pub progress_handler: isize,
    pub realloc: isize,
    pub reset: isize,
    pub result_blob: isize,
    pub result_double: isize,
    pub result_error: isize,
    pub result_error16: isize,
    pub result_int: isize,
    pub result_int64: isize,
    pub result_null: isize,
    pub result_text: isize,
    pub result_text16: isize,
    pub result_text16be: isize,
    pub result_text16le: isize,
    pub result_value: isize,
    pub rollback_hook: isize,
    pub set_authorizer: isize,
    pub set_auxdata: isize,
    pub xsnprintf: isize,
    pub step: isize,
    pub table_column_metadata: isize,
    pub thread_cleanup: isize,
    pub total_changes: isize,
    pub trace: isize,
    pub transfer_bindings: isize,
    pub update_hook: isize,
    pub user_data: isize,
    pub value_blob: isize,
    pub value_bytes: isize,
    pub value_bytes16: isize,
    pub value_double: isize,
    pub value_int: isize,
    pub value_int64: isize,
    pub value_numeric_type: isize,
    pub value_text: isize,
    pub value_text16: isize,
    pub value_text16be: isize,
    pub value_text16le: isize,
    pub value_type: isize,
    pub vmprintf: isize,
    pub overload_function: isize,
    pub prepare_v2: isize,
    pub prepare16_v2: isize,
    pub clear_bindings: isize,
    pub create_module_v2: isize,
    pub bind_zeroblob: isize,
    pub blob_bytes: isize,
    pub blob_close: isize,
    pub blob_open: isize,
    pub blob_read: isize,
    pub blob_write: isize,
    pub create_collation_v2: isize,
    pub file_control: isize,
    pub memory_highwater: isize,
    pub memory_used: isize,
    pub mutex_alloc: isize,
    pub mutex_enter: isize,
    pub mutex_free: isize,
    pub mutex_leave: isize,
    pub mutex_try: isize,
    pub open_v2: isize,
    pub release_memory: isize,
    pub result_error_nomem: isize,
    pub result_error_toobig: isize,
    pub sleep: isize,
    pub soft_heap_limit: isize,
    pub vfs_find: isize,
    pub vfs_register: isize,
    pub vfs_unregister: isize,
    pub xthreadsafe: isize,
    pub result_zeroblob: isize,
    pub result_error_code: isize,
    pub test_control: isize,
    pub randomness: isize,
    pub context_db_handle: isize,
    pub extended_result_codes: isize,
    pub limit: isize,
    pub next_stmt: isize,
    pub sql: isize,
    pub status: isize,
    pub backup_finish: isize,
    pub backup_init: isize,
    pub backup_pagecount: isize,
    pub backup_remaining: isize,
    pub backup_step: isize,
    pub compileoption_get: isize,
    pub compileoption_used: isize,
    pub create_function_v2: isize,
    pub db_config: isize,
    pub db_mutex: isize,
    pub db_status: isize,
    pub extended_errcode: isize,
    pub log: isize,
    pub soft_heap_limit64: isize,
    pub sourceid: isize,
    pub stmt_status: isize,
    pub strnicmp: isize,
    pub unlock_notify: isize,
    pub wal_autocheckpoint: isize,
    pub wal_checkpoint: isize,
    pub wal_hook: isize,
    pub blob_reopen: isize,
    pub vtab_config: isize,
    pub vtab_on_conflict: isize,
    pub close_v2: isize,
    pub db_filename: isize,
    pub db_readonly: isize,
    pub db_release_memory: isize,
    pub errstr: isize,
    pub stmt_busy: isize,
    pub stmt_readonly: isize,
    pub stricmp: isize,
    pub uri_boolean: isize,
    pub uri_int64: isize,
    pub uri_parameter: isize,
    pub xvsnprintf: isize,
    pub wal_checkpoint_v2: isize,
    pub auto_extension: isize,
    pub bind_blob64: isize,
    pub bind_text64: isize,
    pub cancel_auto_extension: isize,
    pub load_extension: isize,
    pub malloc64: isize,
    pub msize: isize,
    pub realloc64: isize,
    pub reset_auto_extension: isize,
    pub result_blob64: isize,
    pub result_text64: isize,
    pub strglob: isize,
    pub value_dup: isize,
    pub value_free: isize,
    pub result_zeroblob64: isize,
    pub bind_zeroblob64: isize,
    pub value_subtype: isize,
    pub result_subtype: isize,
    pub status64: isize,
    pub strlike: isize,
    pub db_cacheflush: isize,
    pub system_errno: isize,
    pub trace_v2: isize,
    pub expanded_sql: isize,
    pub set_last_insert_rowid: isize,
    pub prepare_v3: isize,
    pub prepare16_v3: isize,
    pub bind_pointer: isize,
    pub result_pointer: isize,
    pub value_pointer: isize,
    pub vtab_nochange: isize,
    pub value_nochange: isize,
    pub vtab_collation: isize,
    pub keyword_count: isize,
    pub keyword_name: isize,
    pub keyword_check: isize,
    pub str_new: isize,
    pub str_finish: isize,
    pub str_appendf: isize,
    pub str_vappendf: isize,
    pub str_append: isize,
    pub str_appendall: isize,
    pub str_appendchar: isize,
    pub str_reset: isize,
    pub str_errcode: isize,
    pub str_length: isize,
    pub str_value: isize,
    pub create_window_function: isize,
    pub normalized_sql: isize,
    pub stmt_isexplain: isize,
    pub value_frombind: isize,
    pub drop_modules: isize,
    pub hard_heap_limit64: isize,
    pub uri_key: isize,
    pub filename_database: isize,
    pub filename_journal: isize,
    pub filename_wal: isize,
    pub create_filename: isize,
    pub free_filename: isize,
    pub database_file_object: isize,
    pub txn_state: isize,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for sqlite3_api_routines {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for sqlite3_api_routines {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
#[cfg(target_arch = "x86")]
pub struct sqlite3_api_routines {
    pub aggregate_context: isize,
    pub aggregate_count: isize,
    pub bind_blob: isize,
    pub bind_double: isize,
    pub bind_int: isize,
    pub bind_int64: isize,
    pub bind_null: isize,
    pub bind_parameter_count: isize,
    pub bind_parameter_index: isize,
    pub bind_parameter_name: isize,
    pub bind_text: isize,
    pub bind_text16: isize,
    pub bind_value: isize,
    pub busy_handler: isize,
    pub busy_timeout: isize,
    pub changes: isize,
    pub close: isize,
    pub collation_needed: isize,
    pub collation_needed16: isize,
    pub column_blob: isize,
    pub column_bytes: isize,
    pub column_bytes16: isize,
    pub column_count: isize,
    pub column_database_name: isize,
    pub column_database_name16: isize,
    pub column_decltype: isize,
    pub column_decltype16: isize,
    pub column_double: isize,
    pub column_int: isize,
    pub column_int64: isize,
    pub column_name: isize,
    pub column_name16: isize,
    pub column_origin_name: isize,
    pub column_origin_name16: isize,
    pub column_table_name: isize,
    pub column_table_name16: isize,
    pub column_text: isize,
    pub column_text16: isize,
    pub column_type: isize,
    pub column_value: isize,
    pub commit_hook: isize,
    pub complete: isize,
    pub complete16: isize,
    pub create_collation: isize,
    pub create_collation16: isize,
    pub create_function: isize,
    pub create_function16: isize,
    pub create_module: isize,
    pub data_count: isize,
    pub db_handle: isize,
    pub declare_vtab: isize,
    pub enable_shared_cache: isize,
    pub errcode: isize,
    pub errmsg: isize,
    pub errmsg16: isize,
    pub exec: isize,
    pub expired: isize,
    pub finalize: isize,
    pub free: isize,
    pub free_table: isize,
    pub get_autocommit: isize,
    pub get_auxdata: isize,
    pub get_table: isize,
    pub global_recover: isize,
    pub interruptx: isize,
    pub last_insert_rowid: isize,
    pub libversion: isize,
    pub libversion_number: isize,
    pub malloc: isize,
    pub mprintf: isize,
    pub open: isize,
    pub open16: isize,
    pub prepare: isize,
    pub prepare16: isize,
    pub profile: isize,
    pub progress_handler: isize,
    pub realloc: isize,
    pub reset: isize,
    pub result_blob: isize,
    pub result_double: isize,
    pub result_error: isize,
    pub result_error16: isize,
    pub result_int: isize,
    pub result_int64: isize,
    pub result_null: isize,
    pub result_text: isize,
    pub result_text16: isize,
    pub result_text16be: isize,
    pub result_text16le: isize,
    pub result_value: isize,
    pub rollback_hook: isize,
    pub set_authorizer: isize,
    pub set_auxdata: isize,
    pub xsnprintf: isize,
    pub step: isize,
    pub table_column_metadata: isize,
    pub thread_cleanup: isize,
    pub total_changes: isize,
    pub trace: isize,
    pub transfer_bindings: isize,
    pub update_hook: isize,
    pub user_data: isize,
    pub value_blob: isize,
    pub value_bytes: isize,
    pub value_bytes16: isize,
    pub value_double: isize,
    pub value_int: isize,
    pub value_int64: isize,
    pub value_numeric_type: isize,
    pub value_text: isize,
    pub value_text16: isize,
    pub value_text16be: isize,
    pub value_text16le: isize,
    pub value_type: isize,
    pub vmprintf: isize,
    pub overload_function: isize,
    pub prepare_v2: isize,
    pub prepare16_v2: isize,
    pub clear_bindings: isize,
    pub create_module_v2: isize,
    pub bind_zeroblob: isize,
    pub blob_bytes: isize,
    pub blob_close: isize,
    pub blob_open: isize,
    pub blob_read: isize,
    pub blob_write: isize,
    pub create_collation_v2: isize,
    pub file_control: isize,
    pub memory_highwater: isize,
    pub memory_used: isize,
    pub mutex_alloc: isize,
    pub mutex_enter: isize,
    pub mutex_free: isize,
    pub mutex_leave: isize,
    pub mutex_try: isize,
    pub open_v2: isize,
    pub release_memory: isize,
    pub result_error_nomem: isize,
    pub result_error_toobig: isize,
    pub sleep: isize,
    pub soft_heap_limit: isize,
    pub vfs_find: isize,
    pub vfs_register: isize,
    pub vfs_unregister: isize,
    pub xthreadsafe: isize,
    pub result_zeroblob: isize,
    pub result_error_code: isize,
    pub test_control: isize,
    pub randomness: isize,
    pub context_db_handle: isize,
    pub extended_result_codes: isize,
    pub limit: isize,
    pub next_stmt: isize,
    pub sql: isize,
    pub status: isize,
    pub backup_finish: isize,
    pub backup_init: isize,
    pub backup_pagecount: isize,
    pub backup_remaining: isize,
    pub backup_step: isize,
    pub compileoption_get: isize,
    pub compileoption_used: isize,
    pub create_function_v2: isize,
    pub db_config: isize,
    pub db_mutex: isize,
    pub db_status: isize,
    pub extended_errcode: isize,
    pub log: isize,
    pub soft_heap_limit64: isize,
    pub sourceid: isize,
    pub stmt_status: isize,
    pub strnicmp: isize,
    pub unlock_notify: isize,
    pub wal_autocheckpoint: isize,
    pub wal_checkpoint: isize,
    pub wal_hook: isize,
    pub blob_reopen: isize,
    pub vtab_config: isize,
    pub vtab_on_conflict: isize,
    pub close_v2: isize,
    pub db_filename: isize,
    pub db_readonly: isize,
    pub db_release_memory: isize,
    pub errstr: isize,
    pub stmt_busy: isize,
    pub stmt_readonly: isize,
    pub stricmp: isize,
    pub uri_boolean: isize,
    pub uri_int64: isize,
    pub uri_parameter: isize,
    pub xvsnprintf: isize,
    pub wal_checkpoint_v2: isize,
    pub auto_extension: isize,
    pub bind_blob64: isize,
    pub bind_text64: isize,
    pub cancel_auto_extension: isize,
    pub load_extension: isize,
    pub malloc64: isize,
    pub msize: isize,
    pub realloc64: isize,
    pub reset_auto_extension: isize,
    pub result_blob64: isize,
    pub result_text64: isize,
    pub strglob: isize,
    pub value_dup: isize,
    pub value_free: isize,
    pub result_zeroblob64: isize,
    pub bind_zeroblob64: isize,
    pub value_subtype: isize,
    pub result_subtype: isize,
    pub status64: isize,
    pub strlike: isize,
    pub db_cacheflush: isize,
    pub system_errno: isize,
    pub trace_v2: isize,
    pub expanded_sql: isize,
    pub set_last_insert_rowid: isize,
    pub prepare_v3: isize,
    pub prepare16_v3: isize,
    pub bind_pointer: isize,
    pub result_pointer: isize,
    pub value_pointer: isize,
    pub vtab_nochange: isize,
    pub value_nochange: isize,
    pub vtab_collation: isize,
    pub keyword_count: isize,
    pub keyword_name: isize,
    pub keyword_check: isize,
    pub str_new: isize,
    pub str_finish: isize,
    pub str_appendf: isize,
    pub str_vappendf: isize,
    pub str_append: isize,
    pub str_appendall: isize,
    pub str_appendchar: isize,
    pub str_reset: isize,
    pub str_errcode: isize,
    pub str_length: isize,
    pub str_value: isize,
    pub create_window_function: isize,
    pub normalized_sql: isize,
    pub stmt_isexplain: isize,
    pub value_frombind: isize,
    pub drop_modules: isize,
    pub hard_heap_limit64: isize,
    pub uri_key: isize,
    pub filename_database: isize,
    pub filename_journal: isize,
    pub filename_wal: isize,
    pub create_filename: isize,
    pub free_filename: isize,
    pub database_file_object: isize,
    pub txn_state: isize,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for sqlite3_api_routines {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for sqlite3_api_routines {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct sqlite3_backup(pub u8);
#[repr(C)]
pub struct sqlite3_blob(pub u8);
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub type sqlite3_callback = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: i32, param2: *mut *mut i8, param3: *mut *mut i8) -> i32>;
#[repr(C)]
pub struct sqlite3_context(pub u8);
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub type sqlite3_destructor_type = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void)>;
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_file {
    pub pMethods: *const sqlite3_io_methods,
}
impl ::core::marker::Copy for sqlite3_file {}
impl ::core::clone::Clone for sqlite3_file {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_index_info {
    pub nConstraint: i32,
    pub aConstraint: *mut sqlite3_index_info_0,
    pub nOrderBy: i32,
    pub aOrderBy: *mut sqlite3_index_info_2,
    pub aConstraintUsage: *mut sqlite3_index_info_1,
    pub idxNum: i32,
    pub idxStr: ::windows_sys::core::PSTR,
    pub needToFreeIdxStr: i32,
    pub orderByConsumed: i32,
    pub estimatedCost: f64,
    pub estimatedRows: i64,
    pub idxFlags: i32,
    pub colUsed: u64,
}
impl ::core::marker::Copy for sqlite3_index_info {}
impl ::core::clone::Clone for sqlite3_index_info {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_index_info_0 {
    pub iColumn: i32,
    pub op: u8,
    pub usable: u8,
    pub iTermOffset: i32,
}
impl ::core::marker::Copy for sqlite3_index_info_0 {}
impl ::core::clone::Clone for sqlite3_index_info_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_index_info_1 {
    pub argvIndex: i32,
    pub omit: u8,
}
impl ::core::marker::Copy for sqlite3_index_info_1 {}
impl ::core::clone::Clone for sqlite3_index_info_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_index_info_2 {
    pub iColumn: i32,
    pub desc: u8,
}
impl ::core::marker::Copy for sqlite3_index_info_2 {}
impl ::core::clone::Clone for sqlite3_index_info_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_io_methods {
    pub iVersion: i32,
    pub xClose: isize,
    pub xRead: isize,
    pub xWrite: isize,
    pub xTruncate: isize,
    pub xSync: isize,
    pub xFileSize: isize,
    pub xLock: isize,
    pub xUnlock: isize,
    pub xCheckReservedLock: isize,
    pub xFileControl: isize,
    pub xSectorSize: isize,
    pub xDeviceCharacteristics: isize,
    pub xShmMap: isize,
    pub xShmLock: isize,
    pub xShmBarrier: isize,
    pub xShmUnmap: isize,
    pub xFetch: isize,
    pub xUnfetch: isize,
}
impl ::core::marker::Copy for sqlite3_io_methods {}
impl ::core::clone::Clone for sqlite3_io_methods {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub type sqlite3_loadext_entry = ::core::option::Option<unsafe extern "system" fn(db: *mut sqlite3, pzerrmsg: *mut *mut i8, pthunk: *const sqlite3_api_routines) -> i32>;
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_mem_methods {
    pub xMalloc: isize,
    pub xFree: isize,
    pub xRealloc: isize,
    pub xSize: isize,
    pub xRoundup: isize,
    pub xInit: isize,
    pub xShutdown: isize,
    pub pAppData: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for sqlite3_mem_methods {}
impl ::core::clone::Clone for sqlite3_mem_methods {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_module {
    pub iVersion: i32,
    pub xCreate: isize,
    pub xConnect: isize,
    pub xBestIndex: isize,
    pub xDisconnect: isize,
    pub xDestroy: isize,
    pub xOpen: isize,
    pub xClose: isize,
    pub xFilter: isize,
    pub xNext: isize,
    pub xEof: isize,
    pub xColumn: isize,
    pub xRowid: isize,
    pub xUpdate: isize,
    pub xBegin: isize,
    pub xSync: isize,
    pub xCommit: isize,
    pub xRollback: isize,
    pub xFindFunction: isize,
    pub xRename: isize,
    pub xSavepoint: isize,
    pub xRelease: isize,
    pub xRollbackTo: isize,
    pub xShadowName: isize,
}
impl ::core::marker::Copy for sqlite3_module {}
impl ::core::clone::Clone for sqlite3_module {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct sqlite3_mutex(pub u8);
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_mutex_methods {
    pub xMutexInit: isize,
    pub xMutexEnd: isize,
    pub xMutexAlloc: isize,
    pub xMutexFree: isize,
    pub xMutexEnter: isize,
    pub xMutexTry: isize,
    pub xMutexLeave: isize,
    pub xMutexHeld: isize,
    pub xMutexNotheld: isize,
}
impl ::core::marker::Copy for sqlite3_mutex_methods {}
impl ::core::clone::Clone for sqlite3_mutex_methods {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct sqlite3_pcache(pub u8);
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_pcache_methods {
    pub pArg: *mut ::core::ffi::c_void,
    pub xInit: isize,
    pub xShutdown: isize,
    pub xCreate: isize,
    pub xCachesize: isize,
    pub xPagecount: isize,
    pub xFetch: isize,
    pub xUnpin: isize,
    pub xRekey: isize,
    pub xTruncate: isize,
    pub xDestroy: isize,
}
impl ::core::marker::Copy for sqlite3_pcache_methods {}
impl ::core::clone::Clone for sqlite3_pcache_methods {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_pcache_methods2 {
    pub iVersion: i32,
    pub pArg: *mut ::core::ffi::c_void,
    pub xInit: isize,
    pub xShutdown: isize,
    pub xCreate: isize,
    pub xCachesize: isize,
    pub xPagecount: isize,
    pub xFetch: isize,
    pub xUnpin: isize,
    pub xRekey: isize,
    pub xTruncate: isize,
    pub xDestroy: isize,
    pub xShrink: isize,
}
impl ::core::marker::Copy for sqlite3_pcache_methods2 {}
impl ::core::clone::Clone for sqlite3_pcache_methods2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_pcache_page {
    pub pBuf: *mut ::core::ffi::c_void,
    pub pExtra: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for sqlite3_pcache_page {}
impl ::core::clone::Clone for sqlite3_pcache_page {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_rtree_geometry {
    pub pContext: *mut ::core::ffi::c_void,
    pub nParam: i32,
    pub aParam: *mut f64,
    pub pUser: *mut ::core::ffi::c_void,
    pub xDelUser: isize,
}
impl ::core::marker::Copy for sqlite3_rtree_geometry {}
impl ::core::clone::Clone for sqlite3_rtree_geometry {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_rtree_query_info {
    pub pContext: *mut ::core::ffi::c_void,
    pub nParam: i32,
    pub aParam: *mut f64,
    pub pUser: *mut ::core::ffi::c_void,
    pub xDelUser: isize,
    pub aCoord: *mut f64,
    pub anQueue: *mut u32,
    pub nCoord: i32,
    pub iLevel: i32,
    pub mxLevel: i32,
    pub iRowid: i64,
    pub rParentScore: f64,
    pub eParentWithin: i32,
    pub eWithin: i32,
    pub rScore: f64,
    pub apSqlParam: *mut *mut sqlite3_value,
}
impl ::core::marker::Copy for sqlite3_rtree_query_info {}
impl ::core::clone::Clone for sqlite3_rtree_query_info {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_snapshot {
    pub hidden: [u8; 48],
}
impl ::core::marker::Copy for sqlite3_snapshot {}
impl ::core::clone::Clone for sqlite3_snapshot {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct sqlite3_stmt(pub u8);
#[repr(C)]
pub struct sqlite3_str(pub u8);
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub type sqlite3_syscall_ptr = ::core::option::Option<unsafe extern "system" fn()>;
#[repr(C)]
pub struct sqlite3_value(pub u8);
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_vfs {
    pub iVersion: i32,
    pub szOsFile: i32,
    pub mxPathname: i32,
    pub pNext: *mut sqlite3_vfs,
    pub zName: ::windows_sys::core::PCSTR,
    pub pAppData: *mut ::core::ffi::c_void,
    pub xOpen: isize,
    pub xDelete: isize,
    pub xAccess: isize,
    pub xFullPathname: isize,
    pub xDlOpen: isize,
    pub xDlError: isize,
    pub xDlSym: isize,
    pub xDlClose: isize,
    pub xRandomness: isize,
    pub xSleep: isize,
    pub xCurrentTime: isize,
    pub xGetLastError: isize,
    pub xCurrentTimeInt64: isize,
    pub xSetSystemCall: isize,
    pub xGetSystemCall: isize,
    pub xNextSystemCall: isize,
}
impl ::core::marker::Copy for sqlite3_vfs {}
impl ::core::clone::Clone for sqlite3_vfs {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_vtab {
    pub pModule: *const sqlite3_module,
    pub nRef: i32,
    pub zErrMsg: ::windows_sys::core::PSTR,
}
impl ::core::marker::Copy for sqlite3_vtab {}
impl ::core::clone::Clone for sqlite3_vtab {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_System_SqlLite'*"]
pub struct sqlite3_vtab_cursor {
    pub pVtab: *mut sqlite3_vtab,
}
impl ::core::marker::Copy for sqlite3_vtab_cursor {}
impl ::core::clone::Clone for sqlite3_vtab_cursor {
    fn clone(&self) -> Self {
        *self
    }
}

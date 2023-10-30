#!/bin/sh -e

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
echo "$SCRIPT_DIR"
cd "$SCRIPT_DIR" || { echo "fatal error" >&2; exit 1; }
cargo clean
mkdir -p "$SCRIPT_DIR/../target" "$SCRIPT_DIR/sqlite3"
export SQLITE3_LIB_DIR="$SCRIPT_DIR/sqlite3"
export SQLITE3_INCLUDE_DIR="$SQLITE3_LIB_DIR"

# Download and extract amalgamation
SQLITE=sqlite-amalgamation-3410200
curl -O https://sqlite.org/2023/$SQLITE.zip
unzip -p "$SQLITE.zip" "$SQLITE/sqlite3.c" > "$SQLITE3_LIB_DIR/sqlite3.c"
unzip -p "$SQLITE.zip" "$SQLITE/sqlite3.h" > "$SQLITE3_LIB_DIR/sqlite3.h"
unzip -p "$SQLITE.zip" "$SQLITE/sqlite3ext.h" > "$SQLITE3_LIB_DIR/sqlite3ext.h"
rm -f "$SQLITE.zip"

# Regenerate bindgen file for sqlite3
rm -f "$SQLITE3_LIB_DIR/bindgen_bundled_version.rs"
cargo update
# Just to make sure there is only one bindgen.rs file in target dir
find "$SCRIPT_DIR/../target" -type f -name bindgen.rs -exec rm {} \;
env LIBSQLITE3_SYS_BUNDLING=1 cargo build --features "buildtime_bindgen session" --no-default-features
find "$SCRIPT_DIR/../target" -type f -name bindgen.rs -exec mv {} "$SQLITE3_LIB_DIR/bindgen_bundled_version.rs" \;

# Sanity checks
cd "$SCRIPT_DIR/.." || { echo "fatal error" >&2; exit 1; }
cargo update
cargo test --features "backup blob chrono functions limits load_extension serde_json trace vtab bundled"
printf '    \e[35;1mFinished\e[0m bundled sqlite3 tests\n'

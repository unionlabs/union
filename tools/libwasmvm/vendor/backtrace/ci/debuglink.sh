#!/bin/bash

# Debuglink tests.
# We build crates/debuglink and then move its debuginfo around
# and test that it can still find the debuginfo.

set -ex

cratedir=`pwd`/crates/debuglink
exefile=crates/debuglink/target/debug/debuglink

# Baseline; no separate debug
cargo build --manifest-path crates/debuglink/Cargo.toml
$exefile $cratedir

# Separate debug in same dir
debugfile1=`dirname $exefile`/debuglink.debug
objcopy --only-keep-debug $exefile $debugfile1
strip -g $exefile
(cd `dirname $exefile` && objcopy --add-gnu-debuglink=debuglink.debug debuglink)
$exefile $cratedir

# Separate debug in .debug subdir
debugfile2=`dirname $exefile`/.debug/debuglink.debug
mkdir -p `dirname $debugfile2`
mv $debugfile1 $debugfile2
$exefile $cratedir

# Separate debug in /usr/lib/debug subdir
debugfile3="/usr/lib/debug/$cratedir/target/debug/debuglink.debug"
mkdir -p `dirname $debugfile3`
mv $debugfile2 $debugfile3
$exefile $cratedir

# Separate debug in /usr/lib/debug/.build-id subdir
id=`readelf -n $exefile | grep '^    Build ID: [0-9a-f]' | cut -b 15-`
idfile="/usr/lib/debug/.build-id/${id:0:2}/${id:2}.debug"
mkdir -p `dirname $idfile`
mv $debugfile3 $idfile
$exefile $cratedir

# Replace idfile with a symlink (this is the usual arrangement)
mv $idfile $debugfile3
ln -s $debugfile3 $idfile
$exefile $cratedir

# Supplementary object file using relative path
dwzfile="/usr/lib/debug/.dwz/debuglink.debug"
mkdir -p `dirname $dwzfile`
cp $debugfile3 $debugfile3.copy
dwz -m $dwzfile -rh $debugfile3 $debugfile3.copy
rm $debugfile3.copy
$exefile $cratedir

# Supplementary object file using build ID
dwzid=`readelf -n $dwzfile | grep '^    Build ID: [0-9a-f]' | cut -b 15-`
dwzidfile="/usr/lib/debug/.build-id/${dwzid:0:2}/${dwzid:2}.debug"
mkdir -p `dirname $dwzidfile`
mv $dwzfile $dwzidfile
$exefile $cratedir
mv $dwzidfile $dwzfile

# Missing debug should fail
mv $debugfile3 $debugfile3.tmp
! $exefile $cratedir
mv $debugfile3.tmp $debugfile3

# Missing dwz should fail
mv $dwzfile $dwzfile.tmp
! $exefile $cratedir
mv $dwzfile.tmp $dwzfile

# Cleanup
rm $idfile $debugfile3 $dwzfile
echo Success

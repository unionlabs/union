#!/usr/bin/env bash

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

if ${trace:-false}
then
	set -x
fi

script_dir=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ed25519jni_jvm_dir="${script_dir}/../jvm"
ed25519jni_rust_dir="${script_dir}/../rust"

# Script to run in order to compile a JAR with the Ed25519 JNI libraries from Rust.
# Assumes SciJava's NativeLoader will be used.
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  nativeDir="${ed25519jni_jvm_dir}/natives/linux_64"
  nativeSuffix="so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  nativeDir="${ed25519jni_jvm_dir}/natives/osx_64"
  nativeSuffix="dylib"
else
  echo "JNI is unsupported on this OS. Exiting."
  exit 1
fi

useDebug="0"
while getopts ":d" opt; do
  case $opt in
    d)
      useDebug="1"
      ;;
  esac
done

# Give priority to release directory, unless a debug flag was passed in.
mkdir -p ${nativeDir}
if [ ${useDebug} -eq "1" ]; then
  mode=debug
else
  mode=release
fi

if [[ -d ${ed25519jni_rust_dir}/target/${mode} ]] ; then
  cp -f ${ed25519jni_rust_dir}/target/${mode}/libed25519jni.a ${nativeDir}
  cp -f ${ed25519jni_rust_dir}/target/${mode}/libed25519jni.${nativeSuffix} ${nativeDir}
else
  echo "Unable to obtain required libed25519jni ${mode} libraries. Exiting."
  exit 1
fi

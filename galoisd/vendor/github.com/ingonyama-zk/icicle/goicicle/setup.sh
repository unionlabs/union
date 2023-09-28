#!/bin/bash

SUDO=''
if [ "$EUID" != 0 ]; then 
  echo "Icicle setup script should be run with root privileges, please run this as root"
  SUDO='sudo'
fi


TARGET_BN254="libbn254.so"
TARGET_BLS12_381="libbls12_381.so"
TARGET_BLS12_377="libbls12_377.so"

MAKE_FAIL=0

$SUDO make $1 || MAKE_FAIL=1

if [ $MAKE_FAIL != 0 ]; then
    echo "make failed, install dependencies and re-run setup script with root privileges"
    exit
fi

TARGET_BN254_PATH=$(dirname "$(find `pwd` -name $TARGET_BN254 -print -quit)")/
TARGET_BLS12_381_PATH=$(dirname "$(find `pwd` -name $TARGET_BLS12_381 -print -quit)")/
TARGET_BLS12_377_PATH=$(dirname "$(find `pwd` -name $TARGET_BLS12_377 -print -quit)")/


if [[ "$TARGET_BLS12_377_PATH" != "" ]]; then
    echo "BLS12_377 found @ $TARGET_BLS12_377_PATH"
    export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:$TARGET_BLS12_377_PATH
fi

if [[ "$TARGET_BN254_PATH" != "" ]]; then
    echo "BN254 found @ $TARGET_BN254_PATH"
    export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:$TARGET_BN254_PATH
fi

if [[ "$TARGET_BLS12_381_PATH" != "" ]]; then
    echo "BLS12_381_PATH found @ $TARGET_BLS12_381_PATH"
    export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:$TARGET_BLS12_381_PATH
fi

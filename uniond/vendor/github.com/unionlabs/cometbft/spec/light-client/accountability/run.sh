#!/bin/sh
#
# The script to run all experiments at once

export SCRIPTS_DIR=~/devl/apalache-tests/scripts
export BUILDS="unstable"
export BENCHMARK=001indinv-apalache
export RUN_SCRIPT=./run-all.sh # alternatively, use ./run-parallel.sh
make -e -f ~/devl/apalache-tests/Makefile.common

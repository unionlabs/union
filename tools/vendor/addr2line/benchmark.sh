#!/bin/bash
if [[ $# -le 1 ]]; then
	echo "Usage: $0 <executable> [<addresses>] REFS..."
	exit 1
fi
target="$1"
shift

addresses=""
if [[ -e "$1" ]]; then
	addresses="$1"
	shift
fi

# path to "us"
# readlink -f, but more portable:
dirname=$(perl -e 'use Cwd "abs_path";print abs_path(shift)' "$(dirname "$0")")

# https://stackoverflow.com/a/2358432/472927
{
	# compile all refs
	pushd "$dirname" > /dev/null
	# if the user has some local changes, preserve them
	nstashed=$(git stash list | wc -l)
	echo "==> Stashing any local modifications"
	git stash --keep-index > /dev/null
	popstash() {
		# https://stackoverflow.com/q/24520791/472927
		if [[ "$(git stash list | wc -l)" -ne "$nstashed" ]]; then
			echo "==> Restoring stashed state"
			git stash pop > /dev/null
		fi
	}
	# if the user has added stuff to the index, abort
	if ! git diff-index --quiet HEAD --; then
		echo "Refusing to overwrite outstanding git changes"
		popstash
		exit 2
	fi
	current=$(git symbolic-ref --short HEAD)
	for ref in "$@"; do
		echo "==> Compiling $ref"
		git checkout -q "$ref"
		commit=$(git rev-parse HEAD)
		fn="target/release/addr2line-$commit"
		if [[ ! -e "$fn" ]]; then
			cargo build --release --example addr2line
			cp target/release/examples/addr2line "$fn"
		fi
		if [[ "$ref" != "$commit" ]]; then
			ln -sfn "addr2line-$commit" target/release/addr2line-"$ref"
		fi
	done
	git checkout -q "$current"
	popstash
	popd > /dev/null

	# get us some addresses to look up
	if [[ -z "$addresses" ]]; then
		echo "==> Looking for benchmarking addresses (this may take a while)"
		addresses=$(mktemp tmp.XXXXXXXXXX)
		objdump -C -x --disassemble -l "$target" \
			| grep -P '0[048]:' \
			| awk '{print $1}' \
			| sed 's/:$//' \
			> "$addresses"
		echo "  -> Addresses stored in $addresses; you should re-use it next time"
	fi

	run() {
		func="$1"
		name="$2"
		cmd="$3"
		args="$4"
		printf "%s\t%s\t" "$name" "$func"
		if [[ "$cmd" =~ llvm-symbolizer ]]; then
			/usr/bin/time -f '%e\t%M' "$cmd" $args -obj="$target" < "$addresses" 2>&1 >/dev/null
		else
			/usr/bin/time -f '%e\t%M' "$cmd" $args -e "$target" < "$addresses" 2>&1 >/dev/null
		fi
	}

	# run without functions
	log1=$(mktemp tmp.XXXXXXXXXX)
	echo "==> Benchmarking"
	run nofunc binutils addr2line >> "$log1"
	#run nofunc elfutils eu-addr2line >> "$log1"
	run nofunc llvm-sym llvm-symbolizer -functions=none >> "$log1"
	for ref in "$@"; do
		run nofunc "$ref" "$dirname/target/release/addr2line-$ref" >> "$log1"
	done
	cat "$log1" | column -t

	# run with functions
	log2=$(mktemp tmp.XXXXXXXXXX)
	echo "==> Benchmarking with -f"
	run func binutils addr2line "-f -i" >> "$log2"
	#run func elfutils eu-addr2line "-f -i"  >> "$log2"
	run func llvm-sym llvm-symbolizer "-functions=linkage -demangle=0" >> "$log2"
	for ref in "$@"; do
		run func "$ref" "$dirname/target/release/addr2line-$ref" "-f -i" >> "$log2"
	done
	cat "$log2" | column -t
	cat "$log2" >> "$log1"; rm "$log2"

	echo "==> Plotting"
	Rscript --no-readline --no-restore --no-save "$dirname/bench.plot.r" < "$log1"

	echo "==> Cleaning up"
	rm "$log1"
	exit 0
}

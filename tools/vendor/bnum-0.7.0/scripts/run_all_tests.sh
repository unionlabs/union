CYAN_COLOR="\033[0;36;1m"
RESET_FORMAT="\033[0m"

test_integer_info () {
	echo "\n${CYAN_COLOR}info${RESET_FORMAT}: running tests with $1-bit test integers..."
}

run_test () {
	test_integer_info "$1"
	RUSTFLAGS="--cfg test_int_bits=\"$1\"" cargo test int --lib --quiet $2
	if [ $? -ne 0 ]
	then
		exit 1
	fi
}

for flags in "" "--all-features"
do
	echo "\n${CYAN_COLOR}info${RESET_FORMAT}: running tests with flags '$flags'..."
	for bits in 64 128
	do
		run_test $bits $flags
	done
done

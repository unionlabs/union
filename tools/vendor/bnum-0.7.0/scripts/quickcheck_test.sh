while true
do
	RUSTFLAGS="--cfg test_int_bits=\"$1\"" cargo test quickcheck_ --quiet --features="$2 rand numtraits nightly"
	clear && printf '\e[3J'
	if [ $? -ne 0 ]
	then
		exit 1
	fi
done
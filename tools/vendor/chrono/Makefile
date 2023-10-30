# this Makefile is mostly for the packaging convenience.
# casual users should use `cargo` to retrieve the appropriate version of Chrono.

CHANNEL=stable

.PHONY: all
all:
	@echo 'Try `cargo build` instead.'

.PHONY: authors
authors:
	echo 'Chrono is mainly written by Kang Seonghoon <public+rust@mearie.org>,' > AUTHORS.txt
	echo 'and also the following people (in ascending order):' >> AUTHORS.txt
	echo >> AUTHORS.txt
	git log --format='%aN <%aE>' | grep -v 'Kang Seonghoon' | sort -u >> AUTHORS.txt

.PHONY: readme README.md
readme: README.md

.PHONY: test
test:
	CHANNEL=$(CHANNEL) ./ci/travis.sh

.PHONY: doc
doc: authors readme
	cargo doc --features 'serde rustc-serialize bincode'

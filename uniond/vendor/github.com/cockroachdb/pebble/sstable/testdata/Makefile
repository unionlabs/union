all: rebuild

.PHONY: rebuild
rebuild: make-table h.txt
	./make-table

h.txt: hamlet-act-1.txt
	cat hamlet-act-1.txt | \
		tr '[:upper:]' '[:lower:]' | \
		grep -o -E '\w+' | \
		sort | \
		uniq -c | \
		awk '{printf "%7s %s\n", $$1, $$2}' \
		> h.txt

make-table: make-table.cc
	g++ -std=c++14 -o make-table make-table.cc -lrocksdb

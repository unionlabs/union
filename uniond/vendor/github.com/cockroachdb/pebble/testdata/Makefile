all: rebuild

.PHONY: rebuild
rebuild: make-db
	rm -fr db-stage-1 && ./make-db 1
	rm -fr db-stage-2 && ./make-db 2
	rm -fr db-stage-3 && ./make-db 3
	rm -fr db-stage-4 && ./make-db 4

make-db: make-db.cc
	g++ -std=c++14 -o make-db make-db.cc -lrocksdb

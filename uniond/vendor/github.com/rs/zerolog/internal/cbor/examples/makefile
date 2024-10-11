all: genLogJSON genLogCBOR

genLogJSON: genLog.go
	go build -o genLogJSON genLog.go 

genLogCBOR: genLog.go
	go build -tags binary_log -o genLogCBOR genLog.go 

clean:
	rm -f genLogJSON genLogCBOR

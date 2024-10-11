## Fuzzers

To run a fuzzer locally, you need [go-fuzz](https://github.com/dvyukov/go-fuzz) installed. 

First build a fuzzing-binary out of the selected package:

```
(cd ./rlp && CGO_ENABLED=0 go-fuzz-build .)
```
That command should generate a `rlp-fuzz.zip` in the `rlp/` directory. If you are already in that directory, you can do

```
[user@work rlp]$ go-fuzz
2019/11/26 13:36:54 workers: 6, corpus: 3 (3s ago), crashers: 0, restarts: 1/0, execs: 0 (0/sec), cover: 0, uptime: 3s
2019/11/26 13:36:57 workers: 6, corpus: 3 (6s ago), crashers: 0, restarts: 1/0, execs: 0 (0/sec), cover: 1054, uptime: 6s
2019/11/26 13:37:00 workers: 6, corpus: 3 (9s ago), crashers: 0, restarts: 1/8358, execs: 25074 (2786/sec), cover: 1054, uptime: 9s
2019/11/26 13:37:03 workers: 6, corpus: 3 (12s ago), crashers: 0, restarts: 1/8497, execs: 50986 (4249/sec), cover: 1054, uptime: 12s
2019/11/26 13:37:06 workers: 6, corpus: 3 (15s ago), crashers: 0, restarts: 1/9330, execs: 74640 (4976/sec), cover: 1054, uptime: 15s
2019/11/26 13:37:09 workers: 6, corpus: 3 (18s ago), crashers: 0, restarts: 1/9948, execs: 99482 (5527/sec), cover: 1054, uptime: 18s
2019/11/26 13:37:12 workers: 6, corpus: 3 (21s ago), crashers: 0, restarts: 1/9428, execs: 122568 (5836/sec), cover: 1054, uptime: 21s
2019/11/26 13:37:15 workers: 6, corpus: 3 (24s ago), crashers: 0, restarts: 1/9676, execs: 145152 (6048/sec), cover: 1054, uptime: 24s
2019/11/26 13:37:18 workers: 6, corpus: 3 (27s ago), crashers: 0, restarts: 1/9855, execs: 167538 (6205/sec), cover: 1054, uptime: 27s
2019/11/26 13:37:21 workers: 6, corpus: 3 (30s ago), crashers: 0, restarts: 1/9645, execs: 192901 (6430/sec), cover: 1054, uptime: 30s
2019/11/26 13:37:24 workers: 6, corpus: 3 (33s ago), crashers: 0, restarts: 1/9967, execs: 219294 (6645/sec), cover: 1054, uptime: 33s

```
Otherwise: 
```
go-fuzz -bin ./rlp/rlp-fuzz.zip
```

### Notes

Once a 'crasher' is found, the fuzzer tries to avoid reporting the same vector twice, so stores the fault in the `suppressions` folder. Thus, if you 
e.g. make changes to fix a bug, you should _remove_ all data from the `suppressions`-folder, to verify that the issue is indeed resolved. 

Also, if you have only one and the same exit-point for multiple different types of test, the suppression can make the fuzzer hide different types of errors. So make
sure that each type of failure is unique (for an example, see the rlp fuzzer, where a counter `i` is used to differentiate between failures: 

```golang
		if !bytes.Equal(input, output) {
			panic(fmt.Sprintf("case %d: encode-decode is not equal, \ninput : %x\noutput: %x", i, input, output))
		}
```


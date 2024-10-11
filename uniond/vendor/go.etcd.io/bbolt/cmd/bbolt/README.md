# Introduction to bbolt command line

`bbolt` provides a command line utility for inspecting and manipulating bbolt database files. To install bbolt command-line please refer [here](https://github.com/etcd-io/bbolt#installing)

**Note**: [etcd](https://github.com/etcd-io/etcd) uses bbolt as its backend storage engine. In this document, we take etcd as an example to demonstrate the usage of bbolt commands. Refer to [install etcd](https://etcd.io/docs/v3.5/install/) for installing etcd.

1. Start a single member etcd cluster with this command below:

    ```bash
    $etcd
    ```

    It will create a directory `default.etcd` by default under current working directory, and the directory structure will look like this:

    ```bash
    $tree default.etcd
    default.etcd
    └── member
        ├── snap
        │   └── db // this is bbolt database file
        └── wal
            └── 0000000000000000-0000000000000000.wal

    3 directories, 2 files
    ```

2. Put some dummy data using [etcdctl](https://github.com/etcd-io/etcd/tree/main/etcdctl).
3. Stop the etcd instance. Note a bbolt database file can only be opened by one read-write process, because it is exclusively locked when opened.

## Usage

- `bbolt command [arguments]`

### help

- help will print information about that command

  ```bash
  $bbolt help

  The commands are:

      version     prints the current version of bbolt
      bench       run synthetic benchmark against bbolt
      buckets     print a list of buckets
      check       verifies integrity of bbolt database
      compact     copies a bbolt database, compacting it in the process
      dump        print a hexadecimal dump of a single page
      get         print the value of a key in a bucket
      info        print basic info
      keys        print a list of keys in a bucket
      help        print this screen
      page        print one or more pages in human readable format
      pages       print list of pages with their types
      page-item   print the key and value of a page item.
      stats       iterate over all pages and generate usage stats
      surgery     perform surgery on bbolt database
  ```

- you can use `help` with any command: `bbolt [command] -h` for more information about command.

## Analyse bbolt database with bbolt command line

### version

- `version` print the current version information of bbolt command-line.
- usage:
  `bbolt version`

  Example:
  
  ```bash
  $bbolt version
  bbolt version: 1.3.7
  Go Version: go1.21.6
  Go OS/Arch: darwin/arm64
  ```

### info

- `info` print the basic information about the given Bbolt database.
- usage:
  `bbolt info [path to the bbolt database]`

    Example:

    ```bash
    $bbolt info ~/default.etcd/member/snap/db
    Page Size: 4096
    ```

  - **note**: page size is given in bytes
  - Bbolt database is using page size of 4KB

### buckets

- `buckets` print a list of buckets of Bbolt database is currently having. Find more information on buckets [here](https://github.com/etcd-io/bbolt#using-buckets)
- usage:
  `bbolt buckets [path to the bbolt database]`

    Example:

    ```bash
    $bbolt buckets ~/default.etcd/member/snap/db
    alarm
    auth
    authRoles
    authUsers
    cluster
    key
    lease
    members
    members_removed
    meta
    ```

  - It means when you start an etcd, it creates these `10` buckets using bbolt database.

### check

- `check` opens a database at a given `[PATH]` and runs an exhaustive check to verify that all pages are accessible or are marked as freed. It also verifies that no pages are double referenced.
- usage:
  `bbolt check [path to the bbolt database]`

    Example:

    ```bash
    $bbolt check ~/default.etcd/member/snap/db
    ok
    ```

  - It returns `ok` as our database file `db` is not corrupted.

### stats

- To gather essential statistics about the bbolt database: `stats` performs an extensive search of the database to track every page reference. It starts at the current meta page and recursively iterates through every accessible bucket.
- usage:
  `bbolt stats [path to the bbolt database]`

  Example:

  ```bash
  $bbolt stats ~/default.etcd/member/snap/db
  Aggregate statistics for 10 buckets

  Page count statistics
      Number of logical branch pages: 0
      Number of physical branch overflow pages: 0
      Number of logical leaf pages: 0
      Number of physical leaf overflow pages: 0
  Tree statistics
      Number of keys/value pairs: 11
      Number of levels in B+tree: 1
  Page size utilization
      Bytes allocated for physical branch pages: 0
      Bytes actually used for branch data: 0 (0%)
      Bytes allocated for physical leaf pages: 0
      Bytes actually used for leaf data: 0 (0%)
  Bucket statistics
      Total number of buckets: 10
      Total number on inlined buckets: 10 (100%)
      Bytes used for inlined buckets: 780 (0%)
  ```

### inspect
- `inspect` inspect the structure of the database.
- Usage: `bbolt inspect [path to the bbolt database]`

  Example:
```bash
$ ./bbolt inspect ~/default.etcd/member/snap/db
{
    "name": "root",
    "keyN": 0,
    "buckets": [
        {
            "name": "alarm",
            "keyN": 0
        },
        {
            "name": "auth",
            "keyN": 2
        },
        {
            "name": "authRoles",
            "keyN": 1
        },
        {
            "name": "authUsers",
            "keyN": 1
        },
        {
            "name": "cluster",
            "keyN": 1
        },
        {
            "name": "key",
            "keyN": 1285
        },
        {
            "name": "lease",
            "keyN": 2
        },
        {
            "name": "members",
            "keyN": 1
        },
        {
            "name": "members_removed",
            "keyN": 0
        },
        {
            "name": "meta",
            "keyN": 3
        }
    ]
}
```

### pages

- Pages prints a table of pages with their type (meta, leaf, branch, freelist).
- The `meta` will store the metadata information of database.
- The `leaf` and `branch` pages will show a key count in the `items` column.
- The `freelist` will show the number of free pages, which are free for writing again.
- The `overflow` column shows the number of blocks that the page spills over into.
- usage:
  `bbolt pages [path to the bbolt database]`

  Example:

  ```bash
  $bbolt pages ~/default.etcd/member/snap/db
  ID       TYPE       ITEMS  OVRFLW
  ======== ========== ====== ======
  0        meta       0
  1        meta       0
  2        free
  3        leaf       10
  4        freelist   2
  5        free
  ```

### page

- Page prints one or more pages in human readable format.
- usage:

  ```bash
  bolt page [path to the bbolt database] pageid [pageid...]
  or: bolt page --all [path to the bbolt database]

  Additional options include:

  --all
    prints all pages (only skips pages that were considered successful overflow pages)
  --format-value=auto|ascii-encoded|hex|bytes|redacted (default: auto)
    prints values (on the leaf page) using the given format
  ```

  Example:

  ```bash
  $bbolt page ~/default.etcd/member/snap/db 3
  Page ID:    3
  Page Type:  leaf
  Total Size: 4096 bytes
  Overflow pages: 0
  Item Count: 10

  "alarm": <pgid=0,seq=0>
  "auth": <pgid=0,seq=0>
  "authRoles": <pgid=0,seq=0>
  "authUsers": <pgid=0,seq=0>
  "cluster": <pgid=0,seq=0>
  "key": <pgid=0,seq=0>
  "lease": <pgid=0,seq=0>
  "members": <pgid=0,seq=0>
  "members_removed": <pgid=0,seq=0>
  "meta": <pgid=0,seq=0>
  ```

  - It prints information of page `page ID: 3`

### page-item

- page-item prints a page item's key and value.
- usage:

  ```bash
  bolt page-item [options] [path to the bbolt database] <pageId> <itemId>
  Additional options include:

      --key-only
          Print only the key
      --value-only
          Print only the value
      --format
          Output format. One of: auto|ascii-encoded|hex|bytes|redacted (default=auto)
  ```

  Example:

  ```bash
  $bbolt page-item --key-only ~/default.etcd/member/snap/db 3 7
  "members"
  ```

  - It returns the key as `--key-only` flag is passed of `pageID: 3` and `itemID: 7`

### dump

- Dump prints a hexadecimal dump of one or more given pages.
- usage:
  `bolt dump [path to the bbolt database] [pageid...]`

### keys

- Print a list of keys in the given bucket.
- usage:

  ```bash
  bolt keys [path to the bbolt database] [BucketName]

  Additional options include:
  --format
    Output format. One of: auto|ascii-encoded|hex|bytes|redacted (default=auto)
  ```

  Example 1:

  ```bash
  $bbolt keys ~/default.etcd/member/snap/db meta
  confState
  consistent_index
  term
  ```

  - It list all the keys in bucket: `meta`

  Example 2:

  ```bash
  $bbolt keys ~/default.etcd/member/snap/db members
  8e9e05c52164694d
  ```

  - It list all the keys in `members` bucket which is a `memberId` of etcd cluster member.
  - In this case we are running a single member etcd cluster, hence only `one memberId` is present. If we would have run a `3` member etcd cluster then it will return a `3 memberId` as `3 cluster members` would have been present in `members` bucket.

### get

- Print the value of the given key in the given bucket.
- usage:
  
  ```bash
  bolt get [path to the bbolt database] [BucketName] [Key]

  Additional options include:
  --format
    Output format. One of: auto|ascii-encoded|hex|bytes|redacted (default=auto)
  --parse-format
    Input format (of key). One of: ascii-encoded|hex (default=ascii-encoded)"
  ```

  Example 1:

  ```bash
  $bbolt get --format=hex ~/default.etcd/member/snap/db meta term
  0000000000000004
  ```

  - It returns the value present in bucket: `meta` for key: `term` in hexadecimal format.

  Example 2:

  ```bash
  $bbolt get ~/default.etcd/member/snap/db members 8e9e05c52164694d
  {"id":10276657743932975437,"peerURLs":["http://localhost:2380"],"name":"default","clientURLs":["http://localhost:2379"]}
  ```

  - It returns the value present in bucket: `members` for key: `8e9e05c52164694d`.

### compact

- Compact opens a database at given `[Source Path]` and walks it recursively, copying keys as they are found from all buckets, to a newly created database at `[Destination Path]`. The original database is left untouched.
- usage:

  ```bash
  bbolt compact [options] -o [Destination Path] [Source Path]

  Additional options include:

  -tx-max-size NUM
    Specifies the maximum size of individual transactions.
    Defaults to 64KB
  ```

  Example:

  ```bash
  $bbolt compact -o ~/db.compact ~/default.etcd/member/snap/db
  16805888 -> 32768 bytes (gain=512.88x)
  ```

  - It will create a compacted database file: `db.compact` at given path.

### bench

- run synthetic benchmark against bbolt database.
- usage:

    ```bash
    Usage:
    -batch-size int

    -blockprofile string

    -count int
            (default 1000)
    -cpuprofile string

    -fill-percent float
            (default 0.5)
    -key-size int
            (default 8)
    -memprofile string

    -no-sync

    -path string

    -profile-mode string
            (default "rw")
    -read-mode string
            (default "seq")
    -value-size int
            (default 32)
    -work

    -write-mode string
            (default "seq")
    ```

    Example:

    ```bash
    $bbolt bench ~/default.etcd/member/snap/db -batch-size 400 -key-size 16
    # Write	68.523572ms	(68.523µs/op)	(14593 op/sec)
    # Read	1.000015152s	(11ns/op)	(90909090 op/sec)
    ```

  - It runs a benchmark with batch size of `400` and with key size of `16` while for others parameters default value is taken.

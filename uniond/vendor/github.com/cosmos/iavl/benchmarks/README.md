# Running Benchmarks

You should run the benchmarks in a container that has all needed support code, one of those is:

```
ghcr.io/notional-labs/cosmos
```

and the source for it is here: https://github.com/notional-labs/containers/blob/master/cosmos/Dockerfile

In


## Setting up the machine

Put the files on the machine and login (all code assumes you are in this directory locally)

```
scp -r setup user@host:
ssh user@host
```

Run the install script (once per machine)

```
cd setup
chmod +x *
sudo ./INSTALL_ROOT.sh
```

## Running the tests

Run the benchmarks in a screen:

```
screen
./RUN_BENCHMARKS.sh
```

Copy them back from your local machine:

```
scp user@host:go/src/github.com/cosmos/iavl/results.txt results.txt
git add results
```

## Running benchmarks with docker

Run the command below to install leveldb and rocksdb from source then run the benchmarks all the dbs (memdb, goleveldb, rocksdb, badgerdb) except boltdb.

replace:
- `baabeetaa` with your repo username and 
- `fix-bencharks` with your branch.

```
docker run --rm -it ubuntu:16.04 /bin/bash -c \
"apt-get update && apt-get install -y curl && \
sh <(curl -s https://raw.githubusercontent.com/baabeetaa/iavl/fix-bencharks/benchmarks/setup/INSTALL_ROOT.sh) && \
sh <(curl -s https://raw.githubusercontent.com/baabeetaa/iavl/fix-bencharks/benchmarks/setup/RUN_BENCHMARKS.sh) fix-bencharks baabeetaa && \
cat ~/iavl/results.txt"
```

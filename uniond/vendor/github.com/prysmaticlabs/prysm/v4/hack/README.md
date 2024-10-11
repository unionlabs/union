# Bash Scripts

This subproject contains useful bash scripts for working with our repository. We have a simple tool that outputs coverage, a simple tool to check for gazelle requirements, update Go protobuf generated files, visibility rules tools for Bazel packages, and more.


## update-go-pbs.sh

This script generates the *.pb.go and *.pb.gw.go files from the *.proto files.
After running `update-go-pbs.sh` keep only the *.pb.go and *pb.gw.go for the protos that have changed before checking in.
*Note*: the generated files may not have imports correctly linted and will need to be fixed to remote associated errors. 
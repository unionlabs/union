#!/bin/bash

# lint modified go files
golangci-lint run --fix --new --fast -c .golangci.yml
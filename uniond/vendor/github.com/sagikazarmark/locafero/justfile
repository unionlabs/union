default:
    just --list

test:
    go test -race -v ./...

lint:
    golangci-lint run

fmt:
    golangci-lint run --fix

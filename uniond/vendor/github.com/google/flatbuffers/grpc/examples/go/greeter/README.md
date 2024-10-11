# Go Greeter example

## Project Structure

    .
    ├── server                   # Server module
    ├── client                   # Client module
    ├── models                   # Flatbuffers models & main grpc code.
    └── README.md

## How to run Server:

- `cd server`

- `go clean`

- `go run main.go`

## How to run Client:

- `cd client`

- `go clean`

- `go run main.go --name NAME`
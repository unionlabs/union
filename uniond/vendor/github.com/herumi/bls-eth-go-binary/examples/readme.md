# Examples

See `sample.go` for basic usage.

To run the example:

```shell
go run examples/sample.go
```

Alternatively, run the example in a minimal `golang-alpine` Docker image:

```shell
docker image build -f examples/alpine.Dockerfile -t herumi-bls-sample ./
docker container run herumi-bls-sample
```


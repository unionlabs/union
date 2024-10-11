## gRPC-Gateway CI testing setup

Contained within is the CI test setup for the Gateway. It runs on Circle CI.

### Whats up with the Dockerfile?

The `Dockerfile` in this folder is used as the build environment when regenerating the files (see CONTRIBUTING.md).
The canonical repository for this Dockerfile is `docker.pkg.github.com/grpc-ecosystem/grpc-gateway/build-env`.

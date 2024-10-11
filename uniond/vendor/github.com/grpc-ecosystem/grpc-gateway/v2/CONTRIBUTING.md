# How to contribute

## Code reviews

All submissions, including submissions by project members, require review.

## I want to regenerate the files after making changes

### Using Docker

It should be as simple as this (run from the root of the repository):

```bash
docker run -v $(pwd):/grpc-gateway -w /grpc-gateway --rm ghcr.io/grpc-ecosystem/grpc-gateway/build-env:1.19 \
    /bin/bash -c 'make install && \
        make clean && \
        make generate'
docker run -itv $(pwd):/grpc-gateway -w /grpc-gateway --entrypoint /bin/bash --rm \
    ghcr.io/grpc-ecosystem/grpc-gateway/build-env:1.19 -c '\
        bazel run :gazelle -- update-repos -from_file=go.mod -to_macro=repositories.bzl%go_repositories && \
        bazel run :gazelle && \
        bazel run :buildifier'
```

You may need to authenticate with GitHub to pull `docker.pkg.github.com/grpc-ecosystem/grpc-gateway/build-env`.
You can do this by following the steps on the [GitHub Package docs](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-to-the-container-registry).

### Using Visual Studio Code dev containers

This repo contains a `devcontainer.json` configuration that sets up the build environment in a container using
[VS Code dev containers](https://code.visualstudio.com/docs/remote/containers). If you're using the dev container,
you can run the commands directly in your terminal:

```sh
$ make install && make clean && make generate
```

```sh
$ bazel run :gazelle -- update-repos -from_file=go.mod -to_macro=repositories.bzl%go_repositories && \
    bazel run :gazelle && \
    bazel run :buildifier
```

Note that the above-listed docker commands will not work in the dev container, since volume mounts from
nested docker container is not possible.

If this has resulted in some file changes in the repo, please ensure you check those in with your merge request.

## Making a release

To make a release, follow these steps:

1. Decide on a release version. The `gorelease` job can
   recommend whether the new release should be a patch or minor release.
1. Tag the release on `main`.
   1. The release can be created using the command line, or also through GitHub's [releases
      UI](https://github.com/grpc-ecosystem/grpc-gateway/releases/new).
   1. If you create a release using the web UI you can publish it as a draft and have it
      reviewed by another maintainer.
   1. Update the release description. Try to include some of the highlights of this release,
      ideally with links to the PRs and crediting the contributors.
1. Update the gorelease job in .github/ci.yaml to point to the new release version.
1. Sit back and pat yourself on the back for a job well done :clap:.

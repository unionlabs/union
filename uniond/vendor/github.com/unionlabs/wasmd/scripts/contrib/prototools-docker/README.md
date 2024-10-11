Prototool Docker Helper
=======================
Docker container for all the protobuf generation... 

Based on the work by @pseudomuto [prototool-docker](https://github.com/charithe/prototool-docker) project:

Installs generators and tools from:

* https://github.com/bufbuild/buf
* https://github.com/grpc-ecosystem
* https://github.com/regen-network/cosmos-proto
* https://github.com/pseudomuto/protoc-gen-doc

### Build
```shell script
docker build -t cosmwasm/prototools-docker -f ./scripts/contrib/prototools-docker/Dockerfile .
```

```shell script
docker run -it  -v $(go list -f "{{ .Dir }}" -m github.com/cosmos/cosmos-sdk):/workspace/cosmos_sdk_dir -v $(pwd):/workspace --workdir /workspace --env COSMOS_SDK_DIR=/cosmos_sdk_dir cosmwasm/prototool-docker sh
```

regenerate:
	cd .. && \
	protoc-min-version --version="3.0.0" --gogoslick_out=\
	Mgoogle/protobuf/duration.proto=github.com/cosmos/gogoproto/types,\
	:. \
	--proto_path=../:../protobuf/:. issue261/issue261.proto

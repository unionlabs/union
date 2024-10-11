regenerate:
	cd .. && \
	protoc-min-version --version="3.0.0" --proto_path=../:../protobuf/:. \
	--gogoslick_out=Mgoogle/protobuf/timestamp.proto=github.com/cosmos/gogoproto/types:. issue262/timefail.proto 

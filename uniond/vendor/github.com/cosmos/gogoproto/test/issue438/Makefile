regenerate:
	cd .. && \
	protoc-min-version --version="3.0.0" --gogo_out=\
	Mgoogle/protobuf/any.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/api.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/duration.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/empty.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/field_mask.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/source_context.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/struct.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/timestamp.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/type.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/wrappers.proto=github.com/cosmos/gogoproto/types,\
	:. --proto_path=../:../protobuf/:. issue438/issue438.proto

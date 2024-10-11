regenerate:
	cd .. && protoc-min-version --version="3.0.0" --gogo_out=\
	Mtest_proto/test.proto=github.com/cosmos/gogoproto/proto/test_proto,\
	Mgoogle/protobuf/any.proto=github.com/cosmos/gogoproto/types:. \
	--proto_path=../protobuf:. proto3_proto/proto3.proto

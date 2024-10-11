# This is the version shipped with google.golang.org/protobuf v1.28.1
VERSION=3.15.3
URL=https://raw.githubusercontent.com/protocolbuffers/protobuf/v${VERSION}/src/google/protobuf

regenerate:

	protoc-min-version \
	--version="3.0.0" \
	--gogotypes_out=\
	Mgoogle/protobuf/api.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/any.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/source_context.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/struct.proto=github.com/cosmos/gogoproto/types,\
	Mgoogle/protobuf/type.proto=github.com/cosmos/gogoproto/types\
	:../types/ \
	-I=. \
	google/protobuf/any.proto \
	google/protobuf/type.proto \
	google/protobuf/empty.proto \
	google/protobuf/api.proto \
	google/protobuf/timestamp.proto \
	google/protobuf/duration.proto \
	google/protobuf/struct.proto \
	google/protobuf/wrappers.proto \
	google/protobuf/field_mask.proto \
	google/protobuf/source_context.proto

	mv ../types/google.golang.org/protobuf/types/known/*/*.pb.go ../types/
	rm -rf ../types/google.golang.org

	# Fix package names, because in gogo they are all in `types`.
	cd ../types && sed -i.bak 's/package anypb/package types/' any.pb.go && rm any.pb.go.bak
	cd ../types && sed -i.bak 's/package apipb/package types/' api.pb.go && rm api.pb.go.bak
	cd ../types && sed -i.bak 's/package durationpb/package types/' duration.pb.go && rm duration.pb.go.bak
	cd ../types && sed -i.bak 's/package emptypb/package types/' empty.pb.go && rm empty.pb.go.bak
	cd ../types && sed -i.bak 's/package fieldmaskpb/package types/' field_mask.pb.go && rm field_mask.pb.go.bak
	cd ../types && sed -i.bak 's/package sourcecontextpb/package types/' source_context.pb.go && rm source_context.pb.go.bak
	cd ../types && sed -i.bak 's/package structpb/package types/' struct.pb.go && rm struct.pb.go.bak
	cd ../types && sed -i.bak 's/package timestamppb/package types/' timestamp.pb.go && rm timestamp.pb.go.bak
	cd ../types && sed -i.bak 's/package typepb/package types/' type.pb.go && rm type.pb.go.bak
	cd ../types && sed -i.bak 's/package wrapperspb/package types/' wrappers.pb.go && rm wrappers.pb.go.bak

update:

	(cd ./google/protobuf && rm descriptor.proto; wget ${URL}/descriptor.proto)
	(cd ./google/protobuf/compiler && rm plugin.proto; wget ${URL}/compiler/plugin.proto)
	(cd ./google/protobuf && rm any.proto; wget ${URL}/any.proto)
	(cd ./google/protobuf && rm empty.proto; wget ${URL}/empty.proto)
	(cd ./google/protobuf && rm timestamp.proto; wget ${URL}/timestamp.proto)
	(cd ./google/protobuf && rm duration.proto; wget ${URL}/duration.proto)
	(cd ./google/protobuf && rm struct.proto; wget ${URL}/struct.proto)
	(cd ./google/protobuf && rm wrappers.proto; wget ${URL}/wrappers.proto)
	(cd ./google/protobuf && rm field_mask.proto; wget ${URL}/field_mask.proto)
	(cd ./google/protobuf && rm api.proto; wget ${URL}/api.proto)
	(cd ./google/protobuf && rm type.proto; wget ${URL}/type.proto)
	(cd ./google/protobuf && rm source_context.proto; wget ${URL}/source_context.proto)


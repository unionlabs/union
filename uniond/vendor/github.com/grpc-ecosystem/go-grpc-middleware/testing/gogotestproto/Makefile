all: test_go

fields_go: fields.proto
	PATH="${GOPATH}/bin:${PATH}" protoc \
	  -I. \
		-I${GOPATH}/src \
		--gogo_out=plugins=grpc:. \
		fields.proto



regenerate:
	cd .. && \
	protoc --proto_path=../:../protobuf/:. \
	--gogo_out=. issue312/issue312.proto 
	(cd events && make regenerate)

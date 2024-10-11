# Running it twice corrects some bugs in clang-format.
for run in {1..2}
do
  clang-format -i include/flatbuffers/* src/*.cpp tests/*.cpp samples/*.cpp grpc/src/compiler/schema_interface.h grpc/tests/*.cpp
done
git checkout include/flatbuffers/reflection_generated.h

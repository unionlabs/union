#!/usr/bin/bash

# Read the date as in the Pacific TZ, with no leading padding
read year month day <<<$(date --date="TZ=\"US/Pacific\"" +'%-y %-m %-d')

version="$year.$month.$day"
version_underscore="$year\_$month\_$day"

echo "Setting Flatbuffers Version to: $version"

echo "Updating include/flatbuffers/base.h..."
sed -i \
  -e "s/\(#define FLATBUFFERS_VERSION_MAJOR \).*/\1$year/" \
  -e "s/\(#define FLATBUFFERS_VERSION_MINOR \).*/\1$month/" \
  -e "s/\(#define FLATBUFFERS_VERSION_REVISION \).*/\1$day/" \
  include/flatbuffers/base.h

echo "Updating CMake\Version.cmake..."
sed -i \
  -e "s/\(set(VERSION_MAJOR \).*/\1$year)/" \
  -e "s/\(set(VERSION_MINOR \).*/\1$month)/" \
  -e "s/\(set(VERSION_PATCH \).*/\1$day)/" \
  CMake/Version.cmake

echo "Updating include/flatbuffers/reflection_generated.h..."
echo "Updating tests/evolution_test/evolution_v1_generated.h..."
echo "Updating tests/evolution_test/evolution_v1_generated.h..."
sed -i \
  -e "s/\(FLATBUFFERS_VERSION_MAJOR == \)[0-9]*\(.*\)/\1$year\2/" \
  -e "s/\(FLATBUFFERS_VERSION_MINOR == \)[0-9]*\(.*\)/\1$month\2/" \
  -e "s/\(FLATBUFFERS_VERSION_REVISION == \)[0-9]*\(.*\)/\1$day\2/" \
  include/flatbuffers/reflection_generated.h \
  tests/evolution_test/evolution_v1_generated.h \
  tests/evolution_test/evolution_v2_generated.h

echo "Updating java/pom.xml..."
xmlstarlet edit --inplace -N s=http://maven.apache.org/POM/4.0.0 \
  --update '//s:project/s:version' --value $version \
  java/pom.xml

echo "Updating package.json..."
sed -i \
  -e "s/\(\"version\": \).*/\1\"$version\",/" \
  package.json

echo "Updating net/FlatBuffers/Google.FlatBuffers.csproj..."
sed -i \
  -e "s/\(<PackageVersion>\).*\(<\/PackageVersion>\)/\1$version\2/" \
  net/FlatBuffers/Google.FlatBuffers.csproj

echo "Updating dart/pubspec.yaml..."
sed -i \
  -e "s/\(version: \).*/\1$version/" \
  dart/pubspec.yaml

echo "Updating python/flatbuffers/_version.py..."
sed -i \
  -e "s/\(__version__ = u\).*/\1\"$version\"/" \
  python/flatbuffers/_version.py

echo "Updating python/setup.py..."
sed -i \
  -e "s/\(version='\).*/\1$version',/" \
  python/setup.py

echo "Updating rust/flatbuffers/Cargo.toml..."
sed -i \
  "s/^version = \".*\"$/version = \"$version\"/g" \
  rust/flatbuffers/Cargo.toml

echo "Updating FlatBuffers.podspec..."
sed -i \
  -e "s/\(s.version\s*= \).*/\1'$version'/" \
  FlatBuffers.podspec

echo "Updating FlatBuffersVersion_X_X_X() version check...."
grep -rl 'FlatBuffersVersion_' *  --exclude=release.sh | xargs -i@ \
  sed -i \
    -e "s/\(FlatBuffersVersion_\).*()/\1$version_underscore()/g" \
    @

echo "Updating FLATBUFFERS_X_X_X() version check...."
grep -rl 'FLATBUFFERS_\d*' *  --exclude=release.sh | xargs -i@ \
  sed -i \
    -e "s/\(FLATBUFFERS_\)[0-9]\{2\}.*()/\1$version_underscore()/g" \
    @

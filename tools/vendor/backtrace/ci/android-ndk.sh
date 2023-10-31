set -ex

ANDROID_ARCH=$1
ANDROID_SDK_VERSION=4333796

mkdir /tmp/android
cd /tmp/android

curl -o android-sdk.zip \
  "https://dl.google.com/android/repository/sdk-tools-linux-${ANDROID_SDK_VERSION}.zip"
unzip -q android-sdk.zip

yes | ./tools/bin/sdkmanager --licenses > /dev/null
./tools/bin/sdkmanager ndk-bundle > /dev/null

./ndk-bundle/build/tools/make_standalone_toolchain.py \
  --arch $ANDROID_ARCH \
  --stl=libc++ \
  --api 21 \
  --install-dir /android-toolchain

cd /tmp
rm -rf android

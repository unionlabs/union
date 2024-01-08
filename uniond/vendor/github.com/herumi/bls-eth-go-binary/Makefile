SRC_DIR?=src
BLS_DIR=$(SRC_DIR)/bls
MCL_DIR=$(BLS_DIR)/mcl
include $(MCL_DIR)/common.mk
ETH_CFLAGS=-DBLS_ETH -DBLS_SWAP_G

UNIT?=8

MIN_CFLAGS=-std=c++03 -O3 -fno-exceptions -fno-rtti -fno-stack-protector -DNDEBUG -DMCL_DONT_USE_OPENSSL -DMCL_LLVM_BMI2=0 -DMCL_USE_LLVM=1 -DMCL_USE_VINT -DMCL_SIZEOF_UNIT=$(UNIT) -DMCL_VINT_FIXED_BUFFER -DMCL_MAX_BIT_SIZE=384 -DCYBOZU_DONT_USE_EXCEPTION -DCYBOZU_DONT_USE_STRING -D_FORTIFY_SOURCE=0 -I$(BLS_DIR)/include -I$(MCL_DIR)/include $(ETH_CFLAGS) $(CFLAGS_USER)
OBJ_DIR=obj
OBJS=$(OBJ_DIR)/bls_c384_256.o $(OBJ_DIR)/fp.o $(OBJ_DIR)/base$(BIT).o

ifeq ($(OS),mingw64)
  _OS=windows
else
  MIN_CFLAGS+=-fPIC
endif
ifeq ($(OS),Linux)
  _OS=linux
endif
ifeq ($(OS),mac)
  _OS=darwin
endif
ifeq ($(OS),mac-m1)
  _OS=darwin
endif
ifeq ($(OS),openbsd)
  _OS=openbsd
endif
ifeq ($(OS),freebsd)
  _OS=freebsd
endif

ifeq ($(CPU),x86-64)
  _ARCH=amd64
  MIN_CFLAGS+=-DMCL_STATIC_CODE -DMCL_DONT_USE_XBYAK
  MCL_STATIC_CODE=1
  OBJS+=$(MCL_DIR)/obj/static_code.o
endif
ifeq ($(CPU),aarch64)
  _ARCH=arm64
endif
ifeq ($(CPU),arm)
  _ARCH=arm
  UNIT=4
endif
ifeq ($(CPU),systemz)
  _ARCH=s390x
endif

LIB_DIR=bls/lib/$(_OS)/$(_ARCH)

all: $(LIB_DIR)/libbls384_256.a

$(LIB_DIR)/libbls384_256.a: $(OBJS)
	-mkdir -p $(LIB_DIR)
	$(AR) $(LIB_DIR)/libbls384_256.a $(OBJS)

$(OBJ_DIR)/fp.o:
	$(CXX) -c -o $(OBJ_DIR)/fp.o $(MCL_DIR)/src/fp.cpp $(MIN_CFLAGS)
$(OBJ_DIR)/base$(BIT).o: $(MCL_DIR)/src/base$(BIT).ll
	$(CXX) -c -o $(OBJ_DIR)/base$(BIT).o $(MCL_DIR)/src/base$(BIT).ll $(MIN_CFLAGS)
$(OBJ_DIR)/bls_c384_256.o:
	$(CXX) -c -o $(OBJ_DIR)/bls_c384_256.o $(BLS_DIR)/src/bls_c384_256.cpp $(MIN_CFLAGS)
$(MCL_DIR)/obj/static_code.o:
	$(MAKE) -C $(MCL_DIR) obj/static_code.o

BASE_LL=$(MCL_DIR)/src/base64.ll $(MCL_DIR)/src/base32.ll

$(MCL_DIR)/src/base64.ll:
	$(MAKE) -C $(MCL_DIR) src/base64.ll

$(MCL_DIR)/src/base32.ll:
	$(MAKE) -C $(MCL_DIR) src/base32.ll BIT=32


ANDROID_TARGET=armeabi-v7a # arm64-v8a x86_64
android: $(BASE_LL)
	@ndk-build -C android/jni NDK_DEBUG=0
	@for target in $(ANDROID_TARGET); do \
		mkdir -p bls/lib/android/$$target; \
		cp android/obj/local/$$target/libbls384_256.a bls/lib/android/$$target/; \
	done

# ios
XCODEPATH=$(shell xcode-select -p)
IOS_CLANG=$(XCODEPATH)/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang
IOS_AR=${XCODEPATH}/Toolchains/XcodeDefault.xctoolchain/usr/bin/ar
PLATFORM?="iPhoneOS"
IOS_MIN_VERSION?=7.0
IOS_CFLAGS=-fembed-bitcode -fno-common -DPIC -fPIC -Dmcl_EXPORTS
IOS_CFLAGS+=-fno-exceptions -fno-rtti -fno-threadsafe-statics -fno-stack-protector -DCYBOZU_DONT_USE_EXCEPTION -DCYBOZU_DONT_USE_STRING
IOS_CFLAGS+=-DMCL_USE_VINT -DMCL_VINT_FIXED_BUFFER -DMCL_DONT_USE_OPENSSL -DMCL_DONT_USE_XBYAK -DMCL_LLVM_BMI2=0 -DMCL_USE_LLVM=1 -std=c++11 -Wall -Wextra -Wformat=2 -Wcast-qual -Wcast-align -Wwrite-strings -Wfloat-equal -Wpointer-arith -O3 -DNDEBUG $(ETH_CFLAGS)
IOS_CFLAGS+=-I $(MCL_DIR)/include -I $(BLS_DIR)/include
IOS_LDFLAGS=-dynamiclib -Wl,-flat_namespace -Wl,-undefined -Wl,suppress
CURVE_BIT?=384_256
IOS_LIB=libbls$(CURVE_BIT).a
IOS_LIBS=ios/armv7/$(IOS_LIB) ios/arm64/$(IOS_LIB) ios/x86_64/$(IOS_LIB) ios/i386/$(IOS_LIB)

ios:
	$(MAKE) each_ios PLATFORM="iPhoneOS" ARCH=armv7 BIT=32 UNIT=4
	$(MAKE) each_ios PLATFORM="iPhoneOS" ARCH=arm64 BIT=64 UNIT=8
	$(MAKE) each_ios PLATFORM="iPhoneSimulator" ARCH=x86_64 BIT=64 UNIT=8
	$(MAKE) each_ios PLATFORM="iPhoneSimulator" ARCH=i386 BIT=32 UNIT=4
	@echo $(IOS_LIBS)
	@mkdir -p bls/lib/ios
	lipo $(IOS_LIBS) -create -output bls/lib/ios/$(IOS_LIB)

each_ios: $(BASE_LL)
	@echo "Building iOS $(ARCH) BIT=$(BIT) UNIT=$(UNIT)"
	$(eval IOS_CFLAGS=$(IOS_CFLAGS) -DMCL_SIZEOF_UNIT=$(UNIT))
	@echo IOS_CFLAGS=$(IOS_CFLAGS)
	$(eval IOS_OUTDIR=ios/$(ARCH))
	$(eval IOS_SDK_PATH=$(XCODEPATH)/Platforms/$(PLATFORM).platform/Developer/SDKs/$(PLATFORM).sdk)
	$(eval IOS_COMMON=-arch $(ARCH) -isysroot $(IOS_SDK_PATH) -mios-version-min=$(IOS_MIN_VERSION))
	@mkdir -p $(IOS_OUTDIR)
	$(IOS_CLANG) $(IOS_COMMON) $(IOS_CFLAGS) -c $(MCL_DIR)/src/fp.cpp -o $(IOS_OUTDIR)/fp.o
	$(IOS_CLANG) $(IOS_COMMON) $(IOS_CFLAGS) -c $(MCL_DIR)/src/base$(BIT).ll -o $(IOS_OUTDIR)/base$(BIT).o
	$(IOS_CLANG) $(IOS_COMMON) $(IOS_CFLAGS) -c $(BLS_DIR)/src/bls_c$(CURVE_BIT).cpp -o $(IOS_OUTDIR)/bls_c$(CURVE_BIT).o
	ar cru $(IOS_OUTDIR)/$(IOS_LIB) $(IOS_OUTDIR)/fp.o $(IOS_OUTDIR)/base$(BIT).o $(IOS_OUTDIR)/bls_c$(CURVE_BIT).o
	ranlib $(IOS_OUTDIR)/$(IOS_LIB)

update:
	cp $(BLS_DIR)/include/bls/bls.h bls/include/bls/.
	cp $(BLS_DIR)/include/bls/bls384_256.h bls/include/bls/.
	cp $(MCL_DIR)/include/mcl/bn.h bls/include/mcl/.
	cp $(MCL_DIR)/include/mcl/bn_c384_256.h bls/include/mcl/.
	cp $(MCL_DIR)/include/mcl/curve_type.h bls/include/mcl/.
	patch -o - -p0 ../bls/ffi/go/bls/mcl.go <patch/mcl.patch > bls/mcl.go
	patch -o - -p0 ../bls/ffi/go/bls/bls.go <patch/bls.patch > bls/bls.go

update_patch:
	-diff -up $(BLS_DIR)/ffi/go/bls/mcl.go bls/mcl.go > patch/mcl.patch
	-diff -up $(BLS_DIR)/ffi/go/bls/bls.go bls/bls.go > patch/bls.patch

clean:
	$(MAKE) -C $(MCL_DIR) clean
	$(MAKE) -C $(BLS_DIR) clean
	$(RM) -rf obj/*.o

.PHONY: android ios each_ios clean

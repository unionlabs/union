LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)

LOCAL_CPP_EXTENSION := .cpp .ll
LOCAL_MODULE := bls384_256

ifeq ($(TARGET_ARCH_ABI),x86_64)
  MY_BIT := 64
#  LOCAL_CPPFLAGS += -fexceptions -fno-rtti
  LOCAL_CPPFLAGS += -DMCL_DONT_USE_XBYAK -fno-exceptions -fno-rtti
endif
ifeq ($(TARGET_ARCH_ABI),arm64-v8a)
  MY_BIT := 64
  LOCAL_CPPFLAGS += -DMCL_DONT_USE_XBYAK -fno-exceptions -fno-rtti
endif
ifeq ($(TARGET_ARCH_ABI),armeabi-v7a)
  MY_BIT := 32
  LOCAL_CPPFLAGS += -DMCL_DONT_USE_XBYAK -fno-exceptions -fno-rtti
endif
ifeq ($(TARGET_ARCH_ABI),x86)
  MY_BIT := 32
  LOCAL_CPPFLAGS += -DMCL_DONT_USE_XBYAK -fno-exceptions -fno-rtti
endif
ifeq ($(MY_BIT),64)
  MY_BASE_LL := $(LOCAL_PATH)/../../src/bls/mcl/src/base64.ll
  LOCAL_CPPFLAGS += -DMCL_SIZEOF_UNIT=8
endif
ifeq ($(MY_BIT),32)
  MY_BASE_LL := $(LOCAL_PATH)/../../src/bls/mcl/src/base32.ll
  LOCAL_CPPFLAGS += -DMCL_SIZEOF_UNIT=4
endif
LOCAL_SRC_FILES :=  $(LOCAL_PATH)/../../src/bls/src/bls_c384_256.cpp $(LOCAL_PATH)/../../src/bls/mcl/src/fp.cpp $(MY_BASE_LL)
LOCAL_C_INCLUDES := $(LOCAL_PATH)/../../src/bls/include $(LOCAL_PATH)/../../src/bls/mcl/include
LOCAL_CPPFLAGS += -O3 -DNDEBUG -fPIC -DMCL_DONT_USE_OPENSSL -DMCL_LLVM_BMI2=0 -DMCL_USE_LLVM=1 -DMCL_USE_VINT -DMCL_VINT_FIXED_BUFFER -DMCL_MAX_BIT_SIZE=384 -DCYBOZU_DONT_USE_EXCEPTION -DCYBOZU_DONT_USE_STRING -std=c++03
LOCAL_CPPFLAGS += -fno-threadsafe-statics

LOCAL_CPPFLAGS += -DBLS_ETH -DBLS_SWAP_G
#LOCAL_LDLIBS := -llog #-Wl,--no-warn-shared-textrel
include $(BUILD_STATIC_LIBRARY)

########################################################################
# Download/copy[/append?] this file to 'jni[/Android.mk]' directory of
# the *target* application. If https://github.com/supranational/blst is
# not configured as a submodule, the repository will be cloned into the
# 'jni' directory. Either way, it would be appropriate to add following
# lines to your .gitignore:
#
#	[app/]jni/blst*
#	[app/]src/supranational/blst

LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_MODULE := blst

LOCAL_SRC_FILES := blst/src/server.c blst/build/assembly.S blst_wrap.cpp

$(LOCAL_PATH)/blst:
	@(cd `dirname $@` && \
	  mod=`git submodule 2>/dev/null | \
	       awk '{ if(match($$2,"/blst$$")) print $$2 }'` && \
	  [ -n "$$mod" ] && ln -s "$$mod" blst \
	 ) || git clone https://github.com/supranational/blst.git $@

$(LOCAL_PATH)/blst/src/server.c $(LOCAL_PATH)/blst/build/assembly.S: $(LOCAL_PATH)/blst

$(LOCAL_PATH)/blst_wrap.cpp: $(LOCAL_PATH)/blst/bindings/blst.swg
	blst_classes=`dirname $@`/../src/supranational/blst && \
	mkdir -p $$blst_classes && \
	swig -c++ -java -package supranational.blst -outdir $$blst_classes -o $@ $<

LOCAL_CFLAGS := -fno-builtin-memcpy -fvisibility=hidden
LOCAL_CPPFLAGS := -fexceptions -I$(LOCAL_PATH)/blst/bindings
LOCAL_LDFLAGS := -Wl,-Bsymbolic

include $(BUILD_SHARED_LIBRARY)

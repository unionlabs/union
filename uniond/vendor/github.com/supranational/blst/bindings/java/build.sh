#!/bin/sh -e

JAVA_PACKAGE=${JAVA_PACKAGE:-supranational.blst}

# figure out if this script was source-d
if [ `dirname "$0"` -ef "$PWD" ]; then
    TOP="$PWD"
elif [ -n "$OLDPWD" -a "$OLDPWD/$0" -ef `basename "$0"` ]; then
    TOP="$PWD"
else
    TOP=`dirname "$0"`
fi

TOP=`(cd "$TOP"/.. && echo $PWD)`

# figure out JAVA_HOME if not set by caller
if [ "x$JAVA_HOME" = "x" -a -x /usr/libexec/java_home ]; then
    JAVA_HOME=`/usr/libexec/java_home 2>/dev/null || true`
fi
if [ "x$JAVA_HOME" = "x" ]; then
    JAVA_HOME=`java -XshowSettings:properties -version 2>&1 | \
               awk 'BEGIN { FS="=[ \t]*" } /java\.home/ { print $2 }'`
fi
if [ ! -d "$JAVA_HOME" ]; then
    echo "JAVA_HOME='$JAVA_HOME' is not a directory" 1>&2
    exit 1
fi
if [ ! -d "$JAVA_HOME"/include ]; then
    if [ -d "$JAVA_HOME"/../include ]; then
        JAVA_HOME=`dirname "$JAVA_HOME"`
    else
        echo "JAVA_HOME='$JAVA_HOME' is not a JDK" 1>&2
        exit 1
    fi
fi
# spot jni_md.h, which is included from jni.h
JNI_MD=`ls "$JAVA_HOME"/include/*/jni_md.h`
JNI_MD=`dirname "$JNI_MD"`

PKG=`echo $JAVA_PACKAGE | tr . /`
mkdir -p $PKG

if [ ! -f blst_wrap.cpp -o "$TOP"/blst.swg -nt blst_wrap.cpp \
                        -o "$TOP"/blst.hpp -nt blst_wrap.cpp \
                        -o "$TOP"/blst.h   -nt blst_wrap.cpp \
                        -o ! -f $PKG/blst.java ]; then
    (set -x; swig -c++ -java -package $JAVA_PACKAGE -outdir $PKG \
                  -o blst_wrap.cpp "$TOP"/blst.swg)
fi

if [ ! -f $PKG/blst.class -o $PKG/blst.java -nt $PKG/blst.class ]; then
    (set -x; "$JAVA_HOME"/bin/javac $PKG/*.java)
fi

# ask blstJNI how does it name the shared library
SO_NAME=$PKG/`"$JAVA_HOME"/bin/java $PKG/blstJNI 2>/dev/null`

LIBBLST_A=libblst.a
if [ -f "$TOP"/libblst.a -a "$TOP"/libblst.a -nt "$TOP"/blst.h ]; then
    LIBBLST_A="$TOP"/libblst.a
elif [ ! -f libblst.a -o "$TOP"/blst.h -nt libblst.a ]; then
    $TOP/../build.sh -fvisibility=hidden "$@"
fi

if [ ! -f $SO_NAME -o blst_wrap.cpp -nt $SO_NAME \
                   -o $LIBBLST_A    -nt $SO_NAME ]; then
    mkdir -p `dirname $SO_NAME`
    case $SO_NAME in
        *.so)   LDFLAGS=${LDFLAGS:--Wl,-Bsymbolic};;
        *.dll)  LDFLAGS=${LDFLAGS:--static-libstdc++};;
    esac
    if [ "x$CXX" = "x" ]; then
        CXX=g++
        which c++ >/dev/null 2>&1 && CXX=c++
    fi
    STD=`${CXX} -dM -E -x c++ /dev/null | \
         awk '{ if($2=="__cplusplus" && $3<"2011") print "-std=c++11"; }'`
    (set -x; ${CXX} ${STD} -shared -o $SO_NAME -fPIC -fvisibility=hidden \
                    -I"$JAVA_HOME"/include -I"$JNI_MD" -I"$TOP" \
                    -O -Wall -Wno-unused-function blst_wrap.cpp \
                    $LIBBLST_A ${LDFLAGS})
fi

if [ ! -f $JAVA_PACKAGE.jar -o $SO_NAME -nt $JAVA_PACKAGE.jar ]; then
    (set -x; "$JAVA_HOME"/bin/jar cf $JAVA_PACKAGE.jar $PKG/*.class $PKG/*/*/*)
fi

#!/bin/sh -e
#
# The script allows to override 'CC', 'CFLAGS' and 'flavour' at command
# line, as well as specify additional compiler flags. For example to
# compile for x32:
#
#	/some/where/build.sh flavour=elf32 -mx32
#
# To cross-compile for mingw/Windows:
#
#	/some/where/build.sh flavour=mingw64 CC=x86_64-w64-mingw32-gcc
#
# In addition script recognizes -shared flag and creates shared library
# alongside libblst.a.
#
# To cross-compile for WebAssembly with Emscripten SDK:
#
#	/some/where/build.sh CROSS_COMPILE=em

[ -d /usr/xpg4/bin ] && PATH=/usr/xpg4/bin:$PATH # Solaris

TOP=`dirname $0`

# if -Werror stands in the way, bypass with -Wno-error on command line,
# or suppress specific one with -Wno-<problematic-warning>
CFLAGS=${CFLAGS:--O -fno-builtin -fPIC -Wall -Wextra -Werror}
PERL=${PERL:-perl}
unset cflags shared

case `uname -s` in
    Darwin)	flavour=macosx
                if (sysctl machdep.cpu.features) 2>/dev/null | grep -q ADX; then
                    cflags="-D__ADX__"
                fi
                ;;
    CYGWIN*)	flavour=mingw64;;
    MINGW*)	flavour=mingw64;;
    *)		flavour=elf;;
esac

while [ "x$1" != "x" ]; do
    case $1 in
        -shared)    shared=1;;
        -*target*)  CFLAGS="$CFLAGS $1";;
        -*)         cflags="$cflags $1";;
        *=*)        eval "$1";;
    esac
    shift
done

if [ "x$CC" = "x" ]; then
    CC=gcc
    which ${CROSS_COMPILE}cc >/dev/null 2>&1 && CC=cc
fi
if which ${CROSS_COMPILE}${CC} >/dev/null 2>&1; then
    CC=${CROSS_COMPILE}${CC}
fi
if [ "x$CROSS_COMPILE" = "x" ]; then
    CROSS_COMPILE=`echo $CC |
                   awk '{ print substr($1,0,match($1,"-(g?cc|clang)$")) }' 2>/dev/null`
    # fix up android prefix...
    CROSS_COMPILE=`echo $CROSS_COMPILE |
                   awk '{ off=match($1,"-android[0-9]+-");
                          if (off) { printf "%sandroid-\n",substr($1,0,off) }
                          else     { print $1 } }'`
fi
NM=${NM:-${CROSS_COMPILE}nm}
AR=${AR:-${CROSS_COMPILE}ar}
OBJCOPY=${OBJCOPY:-${CROSS_COMPILE}objcopy}

if (${CC} ${CFLAGS} -dM -E -x c /dev/null) 2>/dev/null | grep -q x86_64; then
    cflags="$cflags -mno-avx" # avoid costly transitions
    if (grep -q -e '^flags.*\badx\b' /proc/cpuinfo) 2>/dev/null; then
        cflags="-D__ADX__ $cflags"
    fi
fi

CFLAGS="$CFLAGS $cflags"
TMPDIR=${TMPDIR:-/tmp}

rm -f libblst.a
trap '[ $? -ne 0 ] && rm -f libblst.a; rm -f *.o ${TMPDIR}/*.blst.$$' 0

(set -x; ${CC} ${CFLAGS} -c ${TOP}/src/server.c)
(set -x; ${CC} ${CFLAGS} -c ${TOP}/build/assembly.S)
(set -x; ${AR} rc libblst.a *.o)

if [ $shared ]; then
    case $flavour in
        macosx) (set -x; ${CC} -dynamiclib -o libblst.dylib \
                               -all_load libblst.a ${CFLAGS}); exit 0;;
        mingw*) sharedlib=blst.dll
                CFLAGS="${CFLAGS} --entry=DllMain ${TOP}/build/win64/dll.c"
                CFLAGS="${CFLAGS} -nostdlib -lgcc";;
        *)      sharedlib=libblst.so;;
    esac
    echo "{ global: blst_*; BLS12_381_*; local: *; };" > ${TMPDIR}/ld.blst.$$
    (set -x; ${CC} -shared -o $sharedlib \
                   -Wl,--whole-archive,libblst.a,--no-whole-archive ${CFLAGS} \
                   -Wl,-Bsymbolic,--version-script=${TMPDIR}/ld.blst.$$)
fi

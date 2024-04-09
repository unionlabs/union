import sys
import subprocess
import re
import shutil
import os

BLST_WRAP_CPP = sys.argv[2]     # see binding.gyp
BLST_WRAP_CPP_PREBUILD = os.getenv('BLST_WRAP_CPP_PREBUILD')

if BLST_WRAP_CPP_PREBUILD:
    if os.path.isfile(BLST_WRAP_CPP_PREBUILD):
        print("Copying and using '{}'".format(BLST_WRAP_CPP_PREBUILD))
        shutil.copyfile(BLST_WRAP_CPP_PREBUILD, BLST_WRAP_CPP)
        sys.exit(0)
    else:
        print("'{}' not found, attempt to swig".format(BLST_WRAP_CPP_PREBUILD))

try:
    version = subprocess.check_output(["swig", "-version"]).decode('ascii')
    v = re.search(r'(SWIG Version ([0-9]+).*)', version)
    if not v:
        raise OSError(2, "unparseable output from 'swig -version'")
    if int(v.group(2)) >= 4:
        subprocess.check_call(["swig", "-c++", "-javascript",
                                       "-node", "-DV8_VERSION=0x060000",
                                       "-o", BLST_WRAP_CPP, sys.argv[1]])
        if BLST_WRAP_CPP_PREBUILD:
            print("Copying blst_wrap.cpp to '{}'".format(BLST_WRAP_CPP_PREBUILD))
            shutil.copyfile(BLST_WRAP_CPP, BLST_WRAP_CPP_PREBUILD)
    else:
        raise OSError(2, "unsupported " + v.group(1))
    sys.exit(0)
except OSError as e:
    if e.errno != 2:    # not "no such file or directory"
        raise e
    sys.stderr.write("{}\n".format(e))
    sys.exit(e.errno)   # or do something else, say ...

here = os.path.split(sys.argv[0])

version = subprocess.check_output(["node", "--version"]).decode('ascii')
v = re.match(r'^v([0-9]+)', version)
if v:
    maj = int(v.group(1))
    if maj >= 16:
        pass
    elif maj >= 12:
        pre_gen = "blst_wrap.v12.cpp"
    elif maj >= 8:
        pre_gen = "blst_wrap.v8.cpp"

try:
    shutil.copyfile(os.path.join(here[0], pre_gen), BLST_WRAP_CPP)
except NameError:
    sys.stderr.write("unsupported 'node --version': {}".format(version))
    sys.exit(2)         # "no such file or directory"

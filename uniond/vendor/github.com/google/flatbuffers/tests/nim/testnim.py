import glob
import os
import shutil
import subprocess
from pathlib import Path

test_nim_dir = Path(__file__).absolute().parent
test_dir = test_nim_dir.parent


def main():
    try:
        subprocess.check_call("testament --megatest:off all".split())
    finally:
        shutil.rmtree(test_nim_dir / "nimcache")
        shutil.rmtree(test_nim_dir / "testresults")
        for f in glob.glob(str(test_nim_dir / "tests" / "*" / "test")):
            os.remove(f)


if __name__ == "__main__":
    main()

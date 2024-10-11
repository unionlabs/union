#!/usr/bin/env python
# -*- coding: utf-8 -*-
import os
import re
import subprocess
from cpt.packager import ConanMultiPackager



def get_branch():
    try:
        for line in subprocess.check_output("git branch", shell=True).decode().splitlines():
            line = line.strip()
            if line.startswith("*") and " (HEAD detached" not in line:
                return line.replace("*", "", 1).strip()
        return ""
    except Exception:
        pass
    return ""


def get_version():
    version = get_branch()
    match = re.search(r"v(\d+\.\d+\.\d+.*)", version)
    if match:
        return match.group(1)
    return version


def get_reference(username):
    return "flatbuffers/{}@google/stable".format(get_version())


if __name__ == "__main__":
    login_username = os.getenv("CONAN_LOGIN_USERNAME", "aardappel")
    username = os.getenv("CONAN_USERNAME", "google")
    upload = os.getenv("CONAN_UPLOAD", "https://api.bintray.com/conan/aardappel/flatbuffers")
    stable_branch_pattern = os.getenv("CONAN_STABLE_BRANCH_PATTERN", r"v\d+\.\d+\.\d+.*")
    test_folder = os.getenv("CPT_TEST_FOLDER", os.path.join("conan", "test_package"))
    upload_only_when_stable = os.getenv("CONAN_UPLOAD_ONLY_WHEN_STABLE", True)

    builder = ConanMultiPackager(reference=get_reference(username),
                                 username=username,
                                 login_username=login_username,
                                 upload=upload,
                                 stable_branch_pattern=stable_branch_pattern,
                                 upload_only_when_stable=upload_only_when_stable,
                                 test_folder=test_folder)
    builder.add_common_builds(pure_c=False)
    builder.run()

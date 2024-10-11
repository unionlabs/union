Thank you for submitting a PR!

Please delete this standard text once you've created your own description.

If you make changes to any of the code generators (`src/idl_gen*`) be sure to
[build](https://google.github.io/flatbuffers/flatbuffers_guide_building.html) your project, as it will generate code based on the changes. If necessary
the code generation script can be directly run (`scripts/generate_code.py`),
requires Python3. This allows us to better see the effect of the PR.

If your PR includes C++ code, please adhere to the
[Google C++ Style Guide](https://google.github.io/styleguide/cppguide.html),
and don't forget we try to support older compilers (e.g. VS2010, GCC 4.6.3),
so only some C++11 support is available.

For any C++ changes, please make sure to run `sh scripts/clang-format-git.sh`

Include other details as appropriate.

Thanks!

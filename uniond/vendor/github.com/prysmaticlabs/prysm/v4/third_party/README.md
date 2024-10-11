# Third Party Package Patching

This directory includes local patches to third party dependencies we use in Prysm. Sometimes,
we need to make a small change to some dependency for ease of use in Prysm without wanting
to maintain our own fork of the dependency ourselves. Our build tool, [Bazel](https://bazel.build)
allows us to include patches in a seamless manner based on simple diff rules.

This README outlines how patching works in Prysm and an explanation of previously
created patches. 

**Given maintaining a patch can be difficult and tedious,
patches are NOT the recommended way of modifying dependencies in Prysm 
unless really needed**

## Table of Contents

- [Prerequisites](#prerequisites)
- [Creating a Patch](#creating-a-patch)
- [Ethereum APIs Patch](#ethereum-apis-patch)
- [Updating Patches](#updating-patches)

## Prerequisites

**Bazel Installation:**
  - The latest release of [Bazel](https://docs.bazel.build/versions/master/install.html)
  - A modern UNIX operating system (MacOS included)

## Creating a Patch

To create a patch, we need an original version of a dependency which we will refer to as `a`
and the patched version referred to as `b`. 

```
cd /tmp
git clone https://github.com/someteam/somerepo a
git clone https://github.com/someteam/somerepo b && cd b
```
Then, make all your changes in `b` and finally create the diff of all your changes as follows:
```
cd ..
diff -ur --exclude=".git" a b > $GOPATH/src/github.com/prysmaticlabs/prysm/third_party/YOURPATCH.patch
```

Next, we need to tell the Bazel [WORKSPACE](https://github.com/prysmaticlabs/prysm/blob/master/WORKSPACE) to patch the specific dependency.
Here's an example for a patch we use today for the [Ethereum APIs](https://github.com/prysmaticlabs/ethereumapis)
dependency:

```
go_repository(
    name = "com_github_prysmaticlabs_ethereumapis",
    commit = "367ca574419a062ae26818f60bdeb5751a6f538",
    patch_args = ["-p1"],
    patches = [
        "//third_party:com_github_prysmaticlabs_ethereumapis-tags.patch",
    ],
    importpath = "github.com/prysmaticlabs/ethereumapis",
)
```

Now, when used in Prysm, the dependency you patched will have the patched modifications
when you run your code.

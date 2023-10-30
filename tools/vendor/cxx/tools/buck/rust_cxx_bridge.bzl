def rust_cxx_bridge(
        name: str.type,
        src: str.type,
        deps: [str.type] = []):
    native.genrule(
        name = "%s/header" % name,
        out = src + ".h",
        cmd = "cp $(location :%s/generated)/generated.h ${OUT}" % name,
    )

    native.genrule(
        name = "%s/source" % name,
        out = src + ".cc",
        cmd = "cp $(location :%s/generated)/generated.cc ${OUT}" % name,
    )

    native.genrule(
        name = "%s/generated" % name,
        srcs = [src],
        out = ".",
        cmd = "$(exe //:codegen) ${SRCS} -o ${OUT}/generated.h -o ${OUT}/generated.cc",
        type = "cxxbridge",
    )

    native.cxx_library(
        name = name,
        srcs = [":%s/source" % name],
        preferred_linkage = "static",
        exported_deps = deps + [":%s/include" % name],
    )

    native.cxx_library(
        name = "%s/include" % name,
        exported_headers = [":%s/header" % name],
    )

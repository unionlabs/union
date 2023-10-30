def buildscript_args(
        name: str.type,
        package_name: str.type,
        buildscript_rule: str.type,
        outfile: str.type,
        version: str.type,
        cfgs: [str.type] = [],
        features: [str.type] = []):
    _ = package_name
    _ = version
    _ = cfgs
    _ = features
    native.genrule(
        name = name,
        out = outfile,
        cmd = "env RUSTC=rustc TARGET= $(exe %s) | sed -n s/^cargo:rustc-cfg=/--cfg=/p > ${OUT}" % buildscript_rule,
    )

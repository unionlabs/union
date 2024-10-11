BaseImageProvider = provider(fields = ["type"])

def _impl(ctx):
    return BaseImageProvider(type = ctx.build_setting_value)

base_image = rule(
    implementation = _impl,
    build_setting = config.string(flag = True),
)

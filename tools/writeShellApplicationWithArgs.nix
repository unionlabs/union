{ pkgs }:
{ name, runtimeInputs ? [ ], arguments, text }: pkgs.writeShellApplication {
  inherit name;
  runtimeInputs = [ pkgs.argc ] ++ runtimeInputs;
  text = ''
    ${builtins.concatStringsSep "\n" (map (arg: "# @${arguments.${arg}.type or "option"}  --${arg}${
      if builtins.hasAttr "required" arguments.${arg} && builtins.hasAttr "multi" arguments.${arg} then "+"
          else if builtins.hasAttr "required" arguments.${arg} then "!"
          else if builtins.hasAttr "multi" arguments.${arg} then "*"
          else if builtins.hasAttr "rawOpts" arguments.${arg} then "${arguments.${arg}.rawOpts}"
          else ""
        } ${arg.help or ""}") (builtins.attrNames arguments))}
    ${builtins.concatStringsSep "\n" (map (arg: "argc_${arg}=${arguments.${arg}.default or ""}") (builtins.attrNames arguments))}

    eval "$(argc --argc-eval "$0" "$@")"

    ${text}
  '';
}

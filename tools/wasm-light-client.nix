_: {
  perSystem =
    {
      dbg,
      pkgs,
      crane,
      ...
    }:
    let
      inherit (crane.buildWorkspaceMember "tools/parse-wasm-client-type" { })
        parse-wasm-client-type
        ;
    in
    {
      packages = {
        inherit parse-wasm-client-type;
      };

      _module.args.ensure-wasm-client-type =
        { type, file_path }:
        ''
          ${pkgs.lib.getExe parse-wasm-client-type} ${file_path} ${type}
        '';
    };
}

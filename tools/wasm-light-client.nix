{ ... }: {
  perSystem = { dbg, pkgs, crane, ... }:
    let
      parse-wasm-client-type = pkgs.lib.getExe (
        (crane.buildWorkspaceMember {
          crateDirFromRoot = "tools/parse-wasm-client-type";
        }).packages.parse-wasm-client-type
      );
    in
    {
      _module.args.ensure-wasm-client-type = { type, file_path }:
        ''
          ${parse-wasm-client-type} ${file_path} ${type}
        '';
    };
}

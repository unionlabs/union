{ ... }: {
  perSystem = { dbg, pkgs, crane, ... }:
    let
      parse-wasm-client-type = (crane.buildWorkspaceMember {
        crateDirFromRoot = "tools/parse-wasm-client-type";
      }).packages.parse-wasm-client-type;
    in
    {
      packages = {
        parse-wasm-client-type = parse-wasm-client-type;
      };

      _module.args.ensure-wasm-client-type = { type, file_path }:
        ''
          ${pkgs.lib.getExe parse-wasm-client-type} ${file_path} ${type}
        '';
    };
}

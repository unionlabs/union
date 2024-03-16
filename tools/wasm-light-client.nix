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
      _module.args.ensure-wasm-client-size = { max_size, file_path }:
        ''
          file_size=$(stat -c %s "${file_path}")
          max_size_str="${toString max_size}"

          if [ "$file_size" -gt "$max_size_str" ]; then
            echo "Error: File size: $file_size exceeds $max_size_str bytes"
            exit 1
          else
            echo "File size: $file_size bytes"
          fi
        '';
    };
}

# buildPnpmPackage.nix
# ===
# This function allows for easy construct `buildNpmPackage` derivation with
# proper defaults for integrating with PNPM Workspaces.

{
  pnpm,
  nodejs,
  pkgs,
  ...
}:
args@{
  # to derive pname and version
  packageJsonPath,
  # to provision sources additional to monorepo boilerplate
  extraSrcs,
  # workspace project names required for build (e.g. anything with "workspace:*" version declaration)
  pnpmWorkspaces ? [ ],
  hash ? pkgs.lib.fakeHash,
  ...
}:

let
  src =
    with pkgs.lib.fileset;
    (toSource {
      root = ./../..;
      fileset = unions (
        [
          ../../package.json
          ../../pnpm-lock.yaml
          ../../patches
          ../../pnpm-workspace.yaml
          ../../tsconfig.base.json
          ../../tsconfig.build.json
          ../../vitest.setup.ts
          ../../vitest.shared.ts
        ]
        ++ extraSrcs
      );
    });
  packageJson = pkgs.lib.importJSON packageJsonPath;
  pname = packageJson.name;
  inherit (packageJson) version;
  pnpmDeps =
    (pnpm.fetchDeps {
      inherit
        hash
        pname
        pnpmWorkspaces
        src
        version
        ;
      fetcherVersion = 3;
    }).overrideAttrs
      (
        super:
        let
          filterFlags = pkgs.lib.map (p: "--filter=${p}") (super.pnpmWorkspaces or [ ]);
        in
        {
          installPhase = ''
            runHook preInstall
            export HOME=$(mktemp -d)
            mkdir $out
            storePath=$(mktemp -d)
            pnpm config set store-dir $storePath
            pnpm config set side-effects-cache false
            pnpm config set update-notifier false
            pnpm config set manage-package-manager-versions false
            mapfile -t wanted_names <<EOF
            ${pkgs.lib.concatStringsSep "\n" pnpmWorkspaces}
            EOF
            mapfile -t pkgjsons < <(
              find . \
                -path '*/node_modules/*' -prune -o \
                -type f -name package.json -print
            )

            printf 'pkgjsons:\n%s\n' "''\${pkgjsons[@]}"

            declare -a dirs=()

            for pj in ''\${pkgjsons[@]}; do
              name="$(jq -r '.name // empty' "$pj")"
              [[ -z "$name" ]] && continue
              for wanted in ''\${wanted_names[@]}; do
                if [[ "$name" == "$wanted" ]]; then
                  d="$(dirname "$pj")"
                  d="''\${d#./}"
                  dirs+=("$d")
                  break
                fi
              done
            done

            printf 'wanted_names:\n%s\n' ''\${wanted_names[@]}

            declare -A seen=()
            declare -a uniq_dirs=()
            for d in ''\${dirs[@]}; do
              if [[ -n "$d" && -z "''\${seen[$d]:-}" ]]; then
                seen["$d"]=1
                uniq_dirs+=("$d")
              fi
            done

            printf 'uniq_dirs:\n%s\n' ''\${uniq_dirs[@]}

            if [[ ''\${#uniq_dirs[@]} -eq 0 ]]; then
              echo "uniq_dirs empty" >&2
              printf '  - %s\n' ''\${wanted_names[@]} >&2
              exit 1
            fi

            # temp dir cleanup
            TMP_DIR="$(mktemp -d)"
            trap 'rm -rf "$TMP_DIR"' EXIT

            # workspace file is required; TODO: make required input
            if [[ ! -f pnpm-workspace.yaml ]]; then
              echo "pnpm-workspace.yaml not found in $PWD" >&2
              exit 1
            fi

            # disgusting yq mutation
            PKGFILE="$TMP_DIR/_packages.yaml"
            {
              echo "packages:"
              for d in ''\${uniq_dirs[@]}; do
                printf '  - %s\n' "$d"
              done
            } > "$PKGFILE"

            ${pkgs.yq-go}/bin/yq eval-all -i '
              select(fileIndex == 0) *
              {"packages": (select(fileIndex == 1).packages)}
            ' pnpm-workspace.yaml "$PKGFILE"

            echo "Updated .packages to:"
            yq -r '.packages[]' pnpm-workspace.yaml | sed 's/^/  - /'

            # produce pruned lockfile derived from root shared lockfile
            pnpm install \
              --force \
              --lockfile-only \
              --ignore-scripts \
              ${pkgs.lib.escapeShellArgs filterFlags} \
              --registry="$NIX_NPM_REGISTRY" || {
              echo "Lockfile-only pass failed. Ensure root pnpm-lock.yaml matches the workspace."
              exit 1
            }

            # fetch into the store
            pnpm install \
              --force \
              --ignore-scripts \
              --registry="$NIX_NPM_REGISTRY" \
              --frozen-lockfile

            echo 3 > $out/.fetcher-version

            runHook postInstall
          '';
        }
      );
in
pkgs.buildNpmPackage (
  args
  // rec {
    inherit
      pname
      pnpmDeps
      pnpmWorkspaces
      src
      version
      nodejs
      ;
    npmConfigHook = pnpm.configHook;
    npmDeps = pnpmDeps;
    NODE_OPTIONS = "--max-old-space-size=8192"; # Important! prevents out-of-memory errors when building .#app
  }
)

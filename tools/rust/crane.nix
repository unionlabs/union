{ inputs, ... }: {
  perSystem = { pkgs, rust, system, lib, dbg, inputs', ... }:
    let
      # crane = builtins.trace (pkgs.lib.generators.toPretty { } inputs.crane ) inputs.crane;
      inherit (inputs) crane;

      craneLib = crane.lib.${system}.overrideToolchain rust.toolchains.nightly;

      mkChecks = pkgName: checks: lib.mapAttrs'
        (
          name: value: {
            name = "${pkgName}-${name}";
            inherit value;
          }
        )
        checks;

      # root of the repository
      root = builtins.path { name = "root"; path = ../../.; };

      # read the Cargo.toml from the given crate directory into a nix value.
      crateCargoToml = dir: assert builtins.isString dir; lib.trivial.importTOML ../../${dir}/Cargo.toml;

      # For use in source filtering; ensures that a directory and all of it's contents are included
      # in the new filtered source.
      ensureDirectoryIncluded = { path', pathToInclude }:
        # check if the path to include is prefixed with the path, to catch files in sub-folders
        # that we have included:
        # pathToInclude = some/dir
        #    sourcePath = some/dir/file.ext     true
        #    sourcePath = some/dir/sub/file.ext true
        #    sourcePath = some/dir.ext          false
        lib.hasPrefix (pathToInclude + "/") path'
        # check if the path is prefixed with the path to include, to ensure that folders aren't
        # preemptively filtered out:
        # pathToInclude = some/dir/sub
        #    sourcePath = some             true
        #    sourcePath = some/dir         true
        #    sourcePath = some/dir/sub/dir false
        || lib.hasPrefix path' pathToInclude;

      buildWorkspaceMember =
        {
          # the directory that contains the Cargo.toml and src/ for the crate,
          # relative to the repository root.
          crateDirFromRoot
        , # additional source filtering, for including non-rust files in the build.
          #
          # additionalSrcFilter :: string -> path -> bool
          #
          # where path is the path of the file, relative to the repository root.
          additionalSrcFilter ? _: _: false
        , # additional source filtering, for including non-rust files for tests.
          #
          # additionalTestSrcFilter :: string -> path -> bool
          #
          # where path is the path of the file, relative to the repository root.
          additionalTestSrcFilter ? _: _: false
        , # extra attributes to be passed to craneLib.cargoNextest.
          cargoTestExtraAttrs ? { }
        , # extra args to be passed to cargo build.
          cargoBuildExtraArgs ? ""
        , # if set to a string, the crate will be built for the specified target and will
          # rebuild the std library. incompatible with `cargoBuildRustToolchain`.
          buildStdTarget ? null
        , # update the toolchain that will be used for cargo build. defaults to
          # rust.toolchains.nightly. incompatible with `buildStdTarget`.
          cargoBuildRustToolchain ? null
        , # rustflags to be passed to cargo build.
          rustflags ? ""
        , # checkPhase to be passed to the cargo build derivation.
          cargoBuildCheckPhase ? null
        , # installPhase to be passed to the cargo build derivation.
          cargoBuildInstallPhase ? null
        , # a suffix to add to the package name.
          pnameSuffix ? ""
          # extra environment variables to pass to the derivation.
        , extraEnv ? {}
        }:
        let
          cratePname = "${crateInfo.pname}${pnameSuffix}";

          cargoBuildRustToolchain' = lib.throwIf
            ((buildStdTarget != null) && (cargoBuildRustToolchain != null))
            "cannot set both buildStdTarget and cargoBuildRustToolchain"
            (
              if (cargoBuildRustToolchain == null)
              then
                (
                  if buildStdTarget == null
                  then rust.toolchains.nightly
                  else rust.mkBuildStdToolchain { target = buildStdTarget; }
                )
              else cargoBuildRustToolchain
            );

          cargoBuild = craneLib.overrideToolchain cargoBuildRustToolchain';

          workspaceCargoToml = lib.trivial.importTOML (root + "/Cargo.toml");
          workspaceCargoLockPath = root + "/Cargo.lock";

          # removes the root store path from the given path, facilitating easier source filtering.
          removeRootStorePath = path:
            let
              root' = (toString root) + "/";
              path' = toString path;
            in
            lib.throwIfNot
              (lib.hasPrefix root' path')
              "path ${path'} does not have the prefix ${root'}"
              (lib.removePrefix root' path');

          # gets all the local (i.e. path) dependencies for a crate, recursively.
          #
          # note that to make this easier, we define all local dependencies as workspace dependencies.
          getWorkspaceDeps = dir:
            # TODO(benluelo): use lib.pipe
            lib.unique
              (lib.flatten (lib.mapAttrsToList
                (name: value:
                  let
                    path = workspaceCargoToml.workspace.dependencies.${name}.path;
                  in
                  (getWorkspaceDeps path) ++ [ path ])
                (lib.filterAttrs
                  (dependency: value:
                    # dep is a workspace dependency...
                    value ? workspace
                    && value.workspace
                    # ...and that workspace dependency is a path dependency
                    && workspaceCargoToml.workspace.dependencies.${dependency} ? path
                    && (builtins.typeOf workspaceCargoToml.workspace.dependencies.${dependency}.path) == "string")
                  (crateCargoToml dir).dependencies
                ))) ++ [ dir ];

          workspaceDepsForCrate = getWorkspaceDeps (lib.throwIfNot
            (builtins.isString crateDirFromRoot)
            "expected crateDirFromRoot to be a string, but it was a ${builtins.typeOf crateDirFromRoot}: ${crateDirFromRoot}"
            crateDirFromRoot);

          crateSrc = lib.cleanSourceWith {
            name = "${cratePname}-source";
            src = root;
            filter = path: type:
              let
                path' = removeRootStorePath path;
              in
              # first filter down to just the cargo source, and any additional files as specified by
                # additional[Test]SrcFilter
              ((craneLib.filterCargoSources path type)
              || (additionalSrcFilter path' type)
              # TODO: only include this filter for tests; maybe by adding to preConfigureHooks?
              || (additionalTestSrcFilter path' type))
              && (
                path' == "Cargo.toml"
                || path' == "Cargo.lock"
                || (
                  builtins.any
                    (depPath: ensureDirectoryIncluded {
                      inherit path';
                      pathToInclude = depPath;
                    })
                    workspaceDepsForCrate
                )
                # Yes, this does need to be filtered twice - once in the original filter so it's included
                # in the cargo sources, and once again so it's included when filtering down to workspace
                # dependencies
                || (additionalSrcFilter path' type)
                # TODO: Only include this filter for tests; maybe by adding to preConfigureHooks?
                || (additionalTestSrcFilter path' type)
              );
          };

          # patch the workspace Cargo.toml to only contain the local dependencies required to build this crate.
          patchedWorkspaceToml =
            let
              patchedCargoToml = (pkgs.formats.toml { }).generate
                "Cargo.toml"
                (lib.recursiveUpdate workspaceCargoToml { workspace.members = workspaceDepsForCrate; });
            in
            # REVIEW: This can maybe be a runCommand?
            # I'm not touching it though
            pkgs.stdenv.mkDerivation {
              name = "${cratePname}-patched-workspace-cargo-toml";
              src = crateSrc;
              buildPhase = ''
                cp -r . $out
                cp ${patchedCargoToml} $out/Cargo.toml
              '';
            };

          crateInfo =
            let
              toml = crateCargoToml crateDirFromRoot;
            in
            {
              pname = toml.package.name;
              version = toml.package.version;
            };

          packageFilterArg = "-p ${crateInfo.pname}";

          crateAttrs = dbg (extraEnv // {
            inherit (crateInfo) pname version;

            # dontUnpack = true;

            src = patchedWorkspaceToml;

            dummySrc = craneLib.mkDummySrc patchedWorkspaceToml;
            # cargoBuildCommand = "${extraEnvStr} cargo build";
            cargoExtraArgs = packageFilterArg;

            buildInputs = [ pkgs.pkg-config pkgs.openssl ] ++ (
              lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
            );

            cargoVendorDir = craneLib.vendorMultipleCargoDeps {
              inherit (craneLib.findCargoFiles crateSrc) cargoConfigs;
              cargoLockList = [
                workspaceCargoLockPath
              ] ++ (lib.optionals (buildStdTarget != null) ([
                "${cargoBuildRustToolchain'.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/Cargo.lock"
              ]));
            };

            doCheck = cargoBuildCheckPhase != null;
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          });

          artifacts = craneLib.buildDepsOnly crateAttrs;

          cargoNextestAttrs =
            builtins.addErrorContext
              "while evaluating `cargoNextestArgs` for crate `${cratePname}`"
              (
                let
                  crateAttrsWithArtifactsNextest = crateAttrs // {
                    doNotLinkInheritedArtifacts = true;
                    cargoArtifacts = artifacts;
                    buildPhaseCargoCommand = "cargo nextest run ${packageFilterArg}";
                  };
                  sharedAttrs = builtins.intersectAttrs crateAttrsWithArtifactsNextest cargoTestExtraAttrs;
                in
                lib.throwIfNot
                  (sharedAttrs == { })
                  "${
                    builtins.concatStringsSep
                    "\n"
                    (builtins.map
                      (attrName: "cargoTestExtraAttrs is overwriting attribute `${attrName}`")
                      (builtins.attrNames sharedAttrs))
                  }\n\nNOTE: if more configuration is needed, update `crane.buildWorkspaceMember`"
                  (crateAttrsWithArtifactsNextest // cargoTestExtraAttrs)
              );

        in
        {
          packages.${cratePname} = cargoBuild.buildPackage (
            crateAttrs // {
              inherit pnameSuffix;
              cargoExtraArgs = "${packageFilterArg} ${cargoBuildExtraArgs}" + (pkgs.lib.optionalString
                (buildStdTarget != null)
                # the leading space is important here!
                " -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target ${buildStdTarget}");
              RUSTFLAGS = rustflags;
            } // (pkgs.lib.optionalAttrs (cargoBuildCheckPhase != null) ({
              installPhaseCommand = cargoBuildInstallPhase;
            })) // (pkgs.lib.optionalAttrs (cargoBuildCheckPhase != null) ({
              checkPhase = cargoBuildCheckPhase;
            })) // (
              if (buildStdTarget == null)
              # if we're not building std, then use the same artifacts as clippy & tests
              then { cargoArtifacts = artifacts; }
              else { }
            )
          );


          checks = mkChecks "${cratePname}" {
            clippy = craneLib.cargoClippy (crateAttrs // {
              cargoArtifacts = artifacts;
            });
            tests = craneLib.cargoNextest cargoNextestAttrs;
          };
        };
    in
    {
      _module.args = {
        crane = {
          lib = craneLib;
          inherit buildWorkspaceMember ensureDirectoryIncluded;
          buildWasmContract = import ./buildWasmContract.nix {
            inherit buildWorkspaceMember crateCargoToml pkgs lib;
          };
        };
      };
    };
}

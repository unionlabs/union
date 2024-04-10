# cspell:ignore tomls
{ inputs, ... }: {
  perSystem = { self', pkgs, rust, system, lib, dbg, inputs', mkCi, ... }:
    let
      inherit (inputs) crane;

      inherit (lib) flatten unique;

      craneLib = crane.lib.${system}.overrideToolchain rust.toolchains.nightly;

      mkChecks = pkgName: checks: lib.mapAttrs'
        (
          name: value: {
            name = "${pkgName}-${name}";
            inherit value;
          }
        )
        checks;

      # get the crane metadata out of the Cargo.toml
      #
      # [package.metadata.crane]
      # include      = ["path1", "path2"]
      # test-include = ["path3", "path4"]
      getCraneMetadata = toml:
        assert builtins.isAttrs toml;
        lib.attrsets.attrByPath [ "package" "metadata" "crane" ] { } toml;

      getExtraIncludes = memberCargoTomls: attr: unique (flatten (map (toml: (getCraneMetadata toml).${attr} or [ ]) (builtins.attrValues memberCargoTomls)));

      # apparently nix doesn't cache calls to builtins.readFile (which importTOML calls internally), so we cache the cargo tomls here
      # this saves ~2-3 minutes in evaluation time
      readMemberCargoTomls = members: builtins.listToAttrs (map
        (dep: lib.attrsets.nameValuePair dep (lib.trivial.importTOML "${root}/${dep}/Cargo.toml"))
        members
      );


      # root of the repository
      root = builtins.path { name = "root"; path = ../../.; };

      # read the Cargo.toml from the given crate directory into a nix value.
      crateCargoToml = dir:
        assert builtins.isString dir;
        lib.trivial.importTOML ../../${dir}/Cargo.toml;

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

      mkCleanSrc =
        { srcFilter, name }: lib.cleanSourceWith {
          name = "${name}-source";
          src = root;
          filter = path: type:
            let
              path' = removeRootStorePath path;
            in
            # first filter down to just the cargo source, and any additional files as specified by srcFilter
            ((craneLib.filterCargoSources path type)
              || (srcFilter path' type));
        };

      workspaceCargoToml = lib.trivial.importTOML (root + "/Cargo.toml");

      workspaceCargoLockPath = root + "/Cargo.lock";

      buildWorkspaceMember =
        {
          # the directory that contains the Cargo.toml and src/ for the crate,
          # relative to the repository root.
          crateDirFromRoot
        , # extra attributes to be passed to craneLib.cargoNextest.
          cargoTestExtraAttrs ? { }
        , # extra args to be passed to cargo build.
          cargoBuildExtraArgs ? ""
        , # extra args to be passed to cargo clippy.
          cargoClippyExtraArgs ? ""
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
        , extraEnv ? { }
        }:
          assert lib.assertMsg
            (
              (buildStdTarget != null -> cargoBuildRustToolchain == null)
              && (cargoBuildRustToolchain != null -> buildStdTarget == null)
            )
            "cannot set both buildStdTarget (${toString buildStdTarget}) and cargoBuildRustToolchain (${toString cargoBuildRustToolchain})";
          let
            cratePname = "${crateInfo.pname}${pnameSuffix}";

            cargoBuildRustToolchain' =
              if (cargoBuildRustToolchain == null)
              then
                (
                  if buildStdTarget == null
                  then rust.toolchains.nightly
                  else rust.mkBuildStdToolchain { target = buildStdTarget; }
                )
              else cargoBuildRustToolchain;

            cargoBuild = craneLib.overrideToolchain cargoBuildRustToolchain';

            # gets all the local (i.e. path) dependencies for a crate, recursively.
            #
            # note that to make this easier, we define all local dependencies as workspace dependencies.
            getWorkspaceDeps = dir:
              let
                go = dir': foundSoFar:
                  lib.pipe ((crateCargoToml dir').dependencies // (crateCargoToml dir').dev-dependencies or { }) [
                    (lib.filterAttrs
                      (dependency: value:
                        # ...and dep is a workspace dependency...
                        (value.workspace or false)
                        # ...and that workspace dependency is a path dependency...
                        && (builtins.hasAttr "path" workspaceCargoToml.workspace.dependencies.${dependency})
                        # ...and that workspace dependency has not been found yet (to prevent infinite recursion)
                        && !(builtins.elem workspaceCargoToml.workspace.dependencies.${dependency}.path foundSoFar)
                      ))
                    (lib.mapAttrsToList
                      (name: value:
                        let
                          path = workspaceCargoToml.workspace.dependencies.${name}.path;
                        in
                        (go path (unique (foundSoFar ++ [ path ]))) ++ [ path ]))
                    (lib.trivial.concat [ dir' ])
                    flatten
                    unique
                  ];
              in
              go dir [ ];

            workspaceDepsForCrate =
              assert lib.assertMsg
                (builtins.isString crateDirFromRoot)
                "expected crateDirFromRoot to be a string, but it was a ${builtins.typeOf crateDirFromRoot}: ${crateDirFromRoot}";
              (getWorkspaceDeps crateDirFromRoot);

            workspaceDepsForCrateCargoTomls = readMemberCargoTomls workspaceDepsForCrate;

            extraIncludePathsForCrate = getExtraIncludes workspaceDepsForCrateCargoTomls "include";
            extraTestIncludePathsForCrate = getExtraIncludes workspaceDepsForCrateCargoTomls "test-include";

            crateSrc =
              let
                isIncluded = path': builtins.elem path'
                  (
                    unique
                      (flatten
                        (map
                          (dep: map
                            (includedPath: "${dep}/${includedPath}")
                            (lib.attrsets.attrByPath
                              [ "package" "include" ]
                              [ ]
                              (workspaceDepsForCrateCargoTomls.${dep})
                            )
                          )
                          workspaceDepsForCrate)
                      )
                  );
                additionalSrcFilter = path': (builtins.any (include: lib.hasPrefix include path') extraIncludePathsForCrate);
                additionalTestSrcFilter = path': (builtins.any (include: lib.hasPrefix include path') extraTestIncludePathsForCrate);
              in
              mkCleanSrc {
                name = cratePname;
                srcFilter = path': type: ((additionalSrcFilter path')
                  # TODO: only include this filter for tests; maybe by adding to preConfigureHooks?
                  || (additionalTestSrcFilter path')
                  || (isIncluded path')
                )
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
                    || (additionalSrcFilter path')
                    # TODO: Only include this filter for tests; maybe by adding to preConfigureHooks?
                    || (additionalTestSrcFilter path')
                    || (isIncluded path')
                );
              };

            # patch the workspace Cargo.toml to only contain the local dependencies required to build this crate.
            patchedWorkspaceToml =
              let
                patchedCargoToml = (pkgs.formats.toml { }).generate
                  "Cargo.toml"
                  (lib.recursiveUpdate workspaceCargoToml {
                    workspace.members = workspaceDepsForCrate;
                  });
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

            crateAttrs = extraEnv // {
              inherit (crateInfo) pname version;

              src = patchedWorkspaceToml;

              dummySrc = craneLib.mkDummySrc patchedWorkspaceToml;

              # defaults to "--all-targets" otherwise, which breaks some stuff
              cargoCheckExtraArgs = "";

              cargoExtraArgs = packageFilterArg;

              buildInputs = [ pkgs.pkg-config pkgs.openssl ] ++ (
                lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
              );

              cargoVendorDir = craneLib.vendorMultipleCargoDeps {
                inherit (craneLib.findCargoFiles crateSrc) cargoConfigs;
                cargoLockList = [
                  workspaceCargoLockPath
                ] ++ (lib.optionals (buildStdTarget != null) ([
                  ./rust-std-Cargo.lock
                ]));
              };

              doCheck = cargoBuildCheckPhase != null;
              PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            };

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
                    (map
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
                cargoExtraArgs = "-j1 ${packageFilterArg} ${cargoBuildExtraArgs}" + (lib.optionalString
                  (buildStdTarget != null)
                  # the leading space is important here!
                  " -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target ${buildStdTarget}");
                RUSTFLAGS = rustflags;
              } // (lib.optionalAttrs (cargoBuildInstallPhase != null) ({
                installPhaseCommand = cargoBuildInstallPhase;
              })) // (lib.optionalAttrs (cargoBuildCheckPhase != null) ({
                checkPhase = cargoBuildCheckPhase;
              })) // (
                if (buildStdTarget == null)
                # if we're not building std, then use the same artifacts as clippy & tests
                then { cargoArtifacts = artifacts; }
                else { }
              )
            );

            checks = mkChecks "${cratePname}" {
              clippy = mkCi (system == "x86_64-linux") (craneLib.cargoClippy (crateAttrs // {
                cargoArtifacts = artifacts;
                cargoClippyExtraArgs = " -- --deny warnings ${cargoClippyExtraArgs}";
              }));
              tests = mkCi (system == "x86_64-linux") (craneLib.cargoNextest cargoNextestAttrs);
            };
          };

      allCargoTomls = builtins.listToAttrs (map (dep: lib.attrsets.nameValuePair dep (lib.trivial.importTOML "${root}/${dep}/Cargo.toml")) workspaceCargoToml.workspace.members);

      cargoWorkspaceSrc =
        let
          allIncludes = unique
            (flatten
              (map
                (dep: map
                  (includedPath: "${dep}/${includedPath}")
                  (lib.attrsets.attrByPath
                    [ "package" "include" ]
                    [ ]
                    (allCargoTomls.${dep})
                  )
                )
                workspaceCargoToml.workspace.members)
            );

          allCraneIncludes =
            dbg (unique ((getExtraIncludes (readMemberCargoTomls workspaceCargoToml.workspace.members) "include") ++
              (getExtraIncludes (readMemberCargoTomls workspaceCargoToml.workspace.members) "test-include")));
        in
        mkCleanSrc {
          name = "cargo-workspace-src";
          srcFilter =
            with { inherit (lib) hasPrefix; };
            path: _type: builtins.any (x: x) (map (include: hasPrefix include path) (allCraneIncludes ++ allIncludes));
        };
    in
    {
      _module.args = {
        crane = {
          lib = craneLib;
          inherit buildWorkspaceMember ensureDirectoryIncluded;
        } //
        (import ./buildWasmContract.nix {
          inherit buildWorkspaceMember crateCargoToml pkgs lib rust craneLib dbg;
        });
      };

      checks.clippy =
        let
          attrs = {
            pname = "workspace-cargo-clippy";
            version = "0.0.0";
            src = cargoWorkspaceSrc;
            cargoClippyExtraArgs = "--workspace --tests";
            SQLX_OFFLINE = true;
          };
        in
        craneLib.cargoClippy (attrs // { cargoArtifacts = craneLib.buildDepsOnly attrs; });

      packages.rust-coverage =
        let
          craneLib = crane.lib.${system}.overrideToolchain rust.toolchains.dev;
        in
        craneLib.cargoLlvmCov {
          pname = "workspace-cargo-llvm-cov";
          version = "0.0.0";
          cargoLlvmCovExtraArgs = lib.concatStringsSep " " [
            "--workspace"
            "--html"
            "--output-dir=$out"
            "--ignore-filename-regex='((nix/store)|(generated))/.+'"
            "--exclude=zerg"
            "--exclude=parse-wasm-client-type"
            "--exclude=protos"
            "--exclude=contracts"
            "--exclude=unionvisor" # TODO: Figure out why unionvisor tests are flakey
            "--exclude=tidy"
            "--exclude=generate-rust-sol-bindings"
            "--exclude=ensure-blocks"
            "--exclude=ucli"
            "--hide-instantiations"
          ];
          SQLX_OFFLINE = true;
          cargoArtifacts = craneLib.buildDepsOnly {
            pname = "workspace-build-deps-only";
            version = "0.0.0";
            cargoExtraArgs = "--locked";
            doCheck = false;

            buildInputs = [ pkgs.pkg-config pkgs.openssl ] ++ (
              lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
            );
            src = cargoWorkspaceSrc;
          };
          preBuild = ''
            cp --no-preserve=mode ${self'.packages.uniond}/bin/uniond $(pwd)/unionvisor/src/testdata/test_init_cmd/bundle/bins/genesis
            echo 'patching testdata'
            patchShebangs $(pwd)/unionvisor/src/testdata
          '';
          ICS23_TEST_SUITE_DATA_DIR = "${inputs.ics23}/testdata";
          ETHEREUM_CONSENSUS_SPECS_DIR = "${inputs.ethereum-consensus-specs}";

          buildInputs = [ pkgs.pkg-config pkgs.openssl ] ++ (
            lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
          );
          src = cargoWorkspaceSrc;
        };
    };
}

# cspell:ignore tomls
{ inputs, ... }: {
  perSystem = { self', pkgs, unstablePkgs, rust, system, lib, dbg, inputs', mkCi, ... }:
    let
      inherit (inputs) crane;

      inherit (lib) flatten unique;

      craneLib = crane.lib.${system}.overrideToolchain rust.toolchains.nightly;

      fs = unstablePkgs.lib.fileset;

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
      # test-include = ["path3", "path4"]
      getCraneMetadata = toml:
        assert builtins.isAttrs toml;
        lib.attrsets.attrByPath [ "package" "metadata" "crane" ] { } toml;

      getExtraIncludes = memberCargoTomls: unique (flatten (map (toml: (getCraneMetadata toml).test-include or [ ]) (builtins.attrValues memberCargoTomls)));
      getIncludes = memberCargoTomls: unique (
        flatten
          (map
            (memberName:
              map
                (include: "${memberName}/${include}")
                (memberCargoTomls.${memberName}.package.include or [ ])
            )
            (builtins.attrNames memberCargoTomls)
          )
      );

      # map a list of paths relative to the root of the repository to absolute paths that can be used with the fileset api.
      #
      # [string] -> [path]
      mkRootPaths = paths: map (path: ../../${path}) paths;

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

      # first filter down to just the cargo source, and any additional files as specified by srcFilter
      mkCleanSrc =
        { workspaceDepsForCrate
        , extraIncludes
        , name
        }: fs.toSource {
          root = ../../.;
          fileset = fs.union
            # unconditionally include...
            (fs.unions (flatten [ ../../rustfmt.toml ../../clippy.toml ../../Cargo.toml ../../Cargo.lock extraIncludes ]))
            # ...and include rust source of workspace deps
            (fs.intersection
              (fs.unions workspaceDepsForCrate)
              (fs.fileFilter
                (file: (file.name == "Cargo.toml") || (builtins.any file.hasExt [ "rs" ]))
                ../../.
              )
            );
        };

      workspaceCargoToml = lib.trivial.importTOML (root + "/Cargo.toml");

      workspaceCargoLockPath = root + "/Cargo.lock";

      buildWorkspaceMember =
        {
          # the directory that contains the Cargo.toml and src/ for the crate,
          # relative to the repository root.
          crateDirFromRoot
        , # extra attributes to be passed to craneLib.cargoTest.
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
                  let
                    dirCargoToml = crateCargoToml dir';
                  in
                  lib.pipe (dirCargoToml.dependencies // dirCargoToml.dev-dependencies or { } // dirCargoToml.build-dependencies or { }) [
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

            includePathsForCrate = getIncludes workspaceDepsForCrateCargoTomls;
            extraTestIncludePathsForCrate = getExtraIncludes workspaceDepsForCrateCargoTomls;

            crateSrc = mkCleanSrc {
              name = cratePname;
              workspaceDepsForCrate = mkRootPaths workspaceDepsForCrate;
              extraIncludes = mkRootPaths (includePathsForCrate ++ extraTestIncludePathsForCrate);
            };

            # patch the workspace Cargo.toml to only contain the local dependencies required to build this crate.
            crateRepoSource =
              let
                patchedCargoToml = (pkgs.formats.toml { }).generate
                  "Cargo.toml"
                  (lib.recursiveUpdate workspaceCargoToml {
                    workspace.members = workspaceDepsForCrate;
                  });
              in
              # REVIEW: This can maybe be a runCommand?
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

              src = crateRepoSource;

              dummySrc = craneLib.mkDummySrc crateRepoSource;

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

            cargoTestAttrs =
              builtins.addErrorContext
                "while evaluating `cargoTestArgs` for crate `${cratePname}`"
                (
                  let
                    crateAttrsWithArtifactsTest = crateAttrs // {
                      doNotLinkInheritedArtifacts = true;
                      cargoArtifacts = artifacts;
                      buildPhaseCargoCommand = "cargo test ${packageFilterArg}";
                    };
                    sharedAttrs = builtins.intersectAttrs crateAttrsWithArtifactsTest cargoTestExtraAttrs;
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
                    (crateAttrsWithArtifactsTest // cargoTestExtraAttrs)
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
                cargoClippyExtraArgs = "--tests -- --deny warnings ${cargoClippyExtraArgs}";
              }));
              tests = mkCi (system == "x86_64-linux") (craneLib.cargoTest cargoTestAttrs);
            };
          };

      allCargoTomls = builtins.listToAttrs (map (dep: lib.attrsets.nameValuePair dep (lib.trivial.importTOML "${root}/${dep}/Cargo.toml")) workspaceCargoToml.workspace.members);

      cargoWorkspaceSrc =
        let
          includePaths = getIncludes allCargoTomls;
          extraTestIncludePaths = getExtraIncludes allCargoTomls;
        in

        mkCleanSrc {
          name = "cargo-workspace-src";
          workspaceDepsForCrate = mkRootPaths workspaceCargoToml.workspace.members;
          extraIncludes = mkRootPaths (includePaths ++ extraTestIncludePaths);
        };
    in
    {
      _module.args = {
        crane = {
          lib = craneLib;
          inherit buildWorkspaceMember cargoWorkspaceSrc;
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

      # FIXME: currently ICE, https://github.com/unionlabs/union/actions/runs/8882618404/job/24387814904
      # packages.rust-coverage =
      #   let
      #     craneLib = crane.lib.${system}.overrideToolchain rust.toolchains.dev;
      #   in
      #   craneLib.cargoLlvmCov {
      #     pname = "workspace-cargo-llvm-cov";
      #     version = "0.0.0";
      #     cargoLlvmCovExtraArgs = lib.concatStringsSep " " [
      #       "--workspace"
      #       "--html"
      #       "--output-dir=$out"
      #       "--ignore-filename-regex='((nix/store)|(generated))/.+'"
      #       "--exclude=zerg"
      #       "--exclude=parse-wasm-client-type"
      #       "--exclude=protos"
      #       "--exclude=contracts"
      #       "--exclude=unionvisor" # TODO: Figure out why unionvisor tests are flakey
      #       "--exclude=tidy"
      #       "--exclude=generate-rust-sol-bindings"
      #       "--exclude=ensure-blocks"
      #       "--exclude=ucli"
      #       "--hide-instantiations"
      #     ];
      #     SQLX_OFFLINE = true;
      #     cargoArtifacts = craneLib.buildDepsOnly {
      #       pname = "workspace-build-deps-only";
      #       version = "0.0.0";
      #       cargoExtraArgs = "--locked";
      #       doCheck = false;

      #       buildInputs = [ pkgs.pkg-config pkgs.openssl ] ++ (
      #         lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
      #       );
      #       src = cargoWorkspaceSrc;
      #     };
      #     preBuild = ''
      #       cp --no-preserve=mode ${self'.packages.uniond}/bin/uniond $(pwd)/unionvisor/src/testdata/test_init_cmd/bundle/bins/genesis
      #       echo 'patching testdata'
      #       patchShebangs $(pwd)/unionvisor/src/testdata
      #     '';
      #     ICS23_TEST_SUITE_DATA_DIR = "${inputs.ics23}/testdata";
      #     ETHEREUM_CONSENSUS_SPECS_DIR = "${inputs.ethereum-consensus-specs}";

      #     buildInputs = [ pkgs.pkg-config pkgs.openssl ] ++ (
      #       lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
      #     );
      #     src = cargoWorkspaceSrc;
      #   };
    };
}

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
        assert lib.assertMsg (builtins.isString dir) "expected string, found ${builtins.typeOf dir} while trying to read cargo.toml (stringified value: ${toString dir})";
        lib.trivial.importTOML ../../${dir}/Cargo.toml;

      # first filter down to just the cargo source, and any additional files as specified by srcFilter
      mkCleanSrc =
        { workspaceDepsForCrate
        , extraIncludes
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
          # relative to the repository root. or a list of multiple crates.
          crateDirFromRoot
          # the pname to use for this derivation if building multiple packages.
        , pname ? null
          # the version to use for this derivation if building multiple packages.
        , version ? null
        , # extra attributes to be passed to craneLib.cargoTest.
          cargoTestExtraAttrs ? { }
        , # extra args to be passed to cargo build.
          cargoBuildExtraArgs ? ""
        , # extra args to be passed to cargo clippy.
          cargoClippyExtraArgs ? ""
        , # extra args to be passed to cargo test.
          cargoTestExtraArgs ? ""
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
          # if true, build without -j1 and --release.
        , dev ? false
        , extraBuildInputs ? [ ]
        , extraNativeBuildInputs ? [ ]
        }:
          assert builtins.isAttrs extraEnv;
          assert builtins.isBool dev;
          assert lib.assertMsg
            (
              (buildStdTarget != null -> cargoBuildRustToolchain == null)
              && (cargoBuildRustToolchain != null -> buildStdTarget == null)
            )
            "cannot set both buildStdTarget (${toString buildStdTarget}) and cargoBuildRustToolchain (${toString cargoBuildRustToolchain})";
          let
            pnameSuffix' = "${pnameSuffix}${lib.optionalString dev "-dev"}";

            # normalize the crate info passed in, such that we can support both single and multiple packages with the same attribute
            processedCrateInfo =
              if (builtins.isList crateDirFromRoot)
              then
                assert builtins.all builtins.isString crateDirFromRoot;
                {
                  crateDirFromRoot' = crateDirFromRoot;
                  pname' = assert builtins.isString pname; pname;
                  version' = assert builtins.isString version; version;
                }
              else if (builtins.isString crateDirFromRoot)
              then
                let
                  cargoToml = ((crateCargoToml crateDirFromRoot).package);
                in
                {
                  crateDirFromRoot' = [ crateDirFromRoot ];
                  version' = cargoToml.version;
                  pname' = cargoToml.name;
                }
              else
                abort "expected crateDirFromRoot to be a string or a list of strings, but it was a ${builtins.typeOf crateDirFromRoot}: ${toString crateDirFromRoot}";

            inherit (processedCrateInfo)
              crateDirFromRoot' pname' version';

            cargoBuildRustToolchain' =
              if (cargoBuildRustToolchain == null)
              then
                (
                  if buildStdTarget == null
                  then rust.toolchains.nightly
                  else rust.mkBuildStdToolchain { targets = [ buildStdTarget ]; }
                )
              else cargoBuildRustToolchain;

            cargoBuild = craneLib.overrideToolchain cargoBuildRustToolchain';

            # gets all the local (i.e. path) dependencies for a crate, recursively.
            #
            # note that to make this easier, we define all local dependencies as workspace dependencies.
            getWorkspaceDeps = dirs:
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
              unique (flatten (builtins.map (dir: go dir [ ]) dirs));

            workspaceDepsForCrate = (getWorkspaceDeps crateDirFromRoot');

            workspaceDepsForCrateCargoTomls = readMemberCargoTomls workspaceDepsForCrate;

            includePathsForCrate = getIncludes workspaceDepsForCrateCargoTomls;
            extraTestIncludePathsForCrate = getExtraIncludes workspaceDepsForCrateCargoTomls;

            crateSrc = mkCleanSrc {
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
                name = "${pname'}-patched-workspace-cargo-toml";
                src = crateSrc;
                buildPhase = ''
                  cp -r . $out
                  cp ${patchedCargoToml} $out/Cargo.toml
                '';
              };

            packageFilterArgs = lib.concatMapStringsSep
              " "
              (dir: "-p ${(crateCargoToml dir).package.name}")
              crateDirFromRoot';

            crateAttrs = extraEnv // {
              pname = pname';
              version = version';

              src = crateRepoSource;

              dummySrc = craneLib.mkDummySrc crateRepoSource;

              # defaults to "--all-targets" otherwise, which breaks some stuff
              cargoCheckExtraArgs = "";

              cargoExtraArgs = packageFilterArgs;

              buildInputs = [ pkgs.pkg-config pkgs.openssl ] ++ (
                lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
              ) ++ extraBuildInputs;

              nativeBuildInputs = extraNativeBuildInputs;

              cargoVendorDir = craneLib.vendorMultipleCargoDeps {
                inherit (craneLib.findCargoFiles crateSrc) cargoConfigs;
                cargoLockList = [
                  workspaceCargoLockPath
                ] ++ (lib.optionals (buildStdTarget != null) ([
                  ./rust-std-Cargo.lock
                ]));
              };

              PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
              CARGO_PROFILE = if dev then "dev" else "release";
            };

            artifacts = craneLib.buildDepsOnly crateAttrs;

            cargoTestAttrs =
              builtins.addErrorContext
                "while evaluating `cargoTestArgs` for crate `${pname'}`"
                (
                  let
                    crateAttrsWithArtifactsTest = crateAttrs // {
                      doNotLinkInheritedArtifacts = true;
                      cargoArtifacts = artifacts;
                      buildPhaseCargoCommand = "cargo test ${packageFilterArgs} ${cargoTestExtraArgs}";
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
            packages."${pname'}${pnameSuffix'}" = cargoBuild.buildPackage (
              crateAttrs // {
                pnameSuffix = dbg pnameSuffix';
                cargoExtraArgs = "${lib.optionalString (!dev) "-j1"} ${packageFilterArgs} ${cargoBuildExtraArgs}" + (lib.optionalString
                  (buildStdTarget != null)
                  # the leading space is important here!
                  " -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target ${buildStdTarget}");
                RUSTFLAGS = rustflags;
                # we don't want to run cargo check/ cargo test on this derivation since we do that in a separate package
                doCheck = false;
                meta =
                  if (builtins.length (crateDirFromRoot') == 1) then {
                    mainProgram = pname';
                  } else { };
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

            checks = mkChecks pname {
              # NOTE: We don't run this on individual crates, since we run clippy on the entire workspace.
              # Left here for reference in case we ever want to reuse this down the line.
              # clippy = mkCi (system == "x86_64-linux") (craneLib.cargoClippy (crateAttrs // {
              #   cargoArtifacts = artifacts;
              #   cargoClippyExtraArgs = "--tests -- --deny warnings ${cargoClippyExtraArgs}";
              # }));
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
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";

            buildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.protobuf pkgs.perl pkgs.gnumake pkgs.systemd ] ++ (
              lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
            );
            nativeBuildInputs = [
              pkgs.clang
            ];
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

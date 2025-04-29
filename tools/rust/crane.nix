# cspell:ignore tomls
{ inputs, ... }:
{
  perSystem =
    args@{
      pkgs,
      rust,
      system,
      dbg,
      mkCi,
      gitRev,
      ...
    }:
    let
      fs = pkgs.lib.fileset;

      writeTOML = (pkgs.formats.toml { }).generate;

      # clean up the lib namespace for what we actually need
      lib = args.lib // {
        inherit (args.lib.attrsets) nameValuePair attrByPath;
        inherit (args.lib.trivial) importTOML concat;
      };

      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (_: rust.toolchains.nightly);

      # get the crane metadata out of the Cargo.toml. returns an empty attrset if the table is not present.
      #
      # [package.metadata.crane]
      # test-include = ["path3", "path4"]
      #
      # sig :: attrs -> attrs;
      getCraneMetadata =
        toml:
        assert builtins.isAttrs toml;
        lib.attrByPath [
          "package"
          "metadata"
          "crane"
        ] { } toml;

      # get any extra test includes specified in the crane metadata.
      #
      # [package.metadata.crane]
      # test-include = ["path3", "path4"]
      getExtraIncludes =
        memberCargoTomls:
        lib.unique (
          lib.flatten (
            map (toml: (getCraneMetadata toml).test-include or [ ]) (builtins.attrValues memberCargoTomls)
          )
        );

      # get any includes specified in package.include, normalized to the repo root.
      #
      # [package]
      # include = [".sqlx", "README.md"]
      #
      # sig :: { string : attrs } -> [string]
      getIncludes =
        memberCargoTomls:
        assert builtins.isAttrs memberCargoTomls;
        lib.unique (
          lib.flatten (
            map (
              memberName:
              map (include: "${memberName}/${include}") (memberCargoTomls.${memberName}.package.include or [ ])
            ) (builtins.attrNames memberCargoTomls)
          )
        );

      # map a list of paths relative to the root of the repository to absolute paths that can be used with the fileset api.
      #
      # [string] -> [path]
      mkRootPaths = map (path: ../../${path});

      # nix doesn't cache calls to b.readFile (which importTOML calls internally), so we cache the cargo tomls here
      # this saves ~2-3 minutes in evaluation time
      #
      # sig :: [string] -> attrs
      readMemberCargoTomls =
        members:
        builtins.listToAttrs (
          map (dep: lib.nameValuePair dep (lib.importTOML "${root}/${dep}/Cargo.toml")) members
        );

      # root of the repository
      #
      # sig :: path
      root = builtins.path {
        name = "root";
        path = ../../.;
      };

      # read the Cargo.toml from the given crate directory into a nix value.
      #
      # sig :: string -> attrs
      crateCargoToml =
        dir:
        assert lib.assertMsg (builtins.isString dir)
          "expected string, found ${builtins.typeOf dir} while trying to read Cargo.toml (stringified value: ${toString dir})";
        lib.importTOML ../../${dir}/Cargo.toml;

      # check whether a list is a list of a specific type.
      #
      # sig :: (any -> bool) -> [any] -> bool
      isListOf = pred: list: builtins.isList list && builtins.all pred list;

      # build a clean source for the specified workspace members and any extra includes.
      mkCleanSrc =
        {
          # [path]
          workspaceMembers,
          # [path]
          extraIncludes,

          # path | derivation
          cargoToml,
          # path | derivation
          cargoLock,

          # bool
          dontRemoveDevDeps ? false,
        }:
        assert isListOf builtins.isString workspaceMembers;
        assert isListOf builtins.isString extraIncludes;
        let
          filteredSrc = fs.toSource {
            root = ../../.;
            fileset =
              fs.union
                # unconditionally include...
                (fs.unions (lib.flatten [ (mkRootPaths extraIncludes) ]))
                # ...and include rust source of workspace deps
                (
                  fs.intersection (fs.unions (mkRootPaths workspaceMembers)) (
                    fs.fileFilter (file: (builtins.any file.hasExt [ "rs" ])) ../../.
                  )
                );
          };
        in
        pkgs.stdenv.mkDerivation {
          name = "clean-workspace-source";
          src = filteredSrc;
          buildInputs = [
            pkgs.tree
          ];
          buildPhase = ''
            tree .

            cp ${cargoLock} ./Cargo.lock
            cp ${cargoToml} ./Cargo.toml

            ${builtins.concatStringsSep "\n\n" (
              lib.mapAttrsToList (
                path: cargoToml:
                let
                  cargoTomlPath = writeTOML "Cargo.toml" (
                    builtins.removeAttrs cargoToml (lib.optionals (!dontRemoveDevDeps) [ "dev-dependencies" ])
                  );
                in
                "cp ${cargoTomlPath} ./${path}/Cargo.toml"
              ) (readMemberCargoTomls workspaceMembers)
            )}

            cp -r . $out
          '';
        };

      # Cargo.toml of the workspace.
      #
      # sig :: string
      workspaceCargoToml = lib.importTOML (root + "/Cargo.toml");

      # TODO: Assert version = 4;
      normalizedCargoLock = builtins.foldl' (acc: p: lib.recursiveUpdate acc p) { } (
        map (package: {
          ${package.name} = {
            ${package.version} =
              package
              // (lib.optionalAttrs (package ? source) (
                let
                  splitSource = lib.splitString "+" package.source;
                  sourceType = builtins.head splitSource;
                  sourceKey = # trim the commit ref if this is a git source
                    # TODO: figure out how this is actually defined to work in the Cargo.lock schema/spec
                    if sourceType == "git" then builtins.head (lib.splitString "#" package.source) else package.source;
                in
                {
                  __source__.${sourceKey} = package;
                }
              ));
          };
        }) (lib.importTOML (root + "/Cargo.lock")).package
      );

      # get a single package entry from the Cargo.lock.
      #
      # sig :: string -> attrs
      getCargoLockPackageEntry =
        depAndVersion:
        let
          split = lib.splitString " " depAndVersion;
          depName = builtins.head split;
          specifiedVersion = builtins.elemAt split 1;
          specifiedSource = builtins.head (builtins.match "[(](.*)[)]" (builtins.elemAt split 2));
          fullDep = normalizedCargoLock.${depName};
        in
        builtins.removeAttrs (
          # dep name is just the dep name (no version or source)
          # this means that this dependency only exists once in the lockfile, and as such only one version subkey will exist
          if ((builtins.length split) == 1) then
            fullDep.${builtins.head (builtins.attrNames fullDep)}
          # dep name is the dep name and a version (no source)
          else if ((builtins.length split) == 2) then
            fullDep.${specifiedVersion}
          # dep name is the dep name, version, and source
          else if ((builtins.length split) == 3) then
            fullDep.${specifiedVersion}.__source__.${specifiedSource}
          else
            throw "???"
        ) [ "__source__" ];

      getAllPackageDependencies =
        packageName:
        let
          go =
            foundSoFar: packageName':
            let
              packageLockEntry = getCargoLockPackageEntry packageName';
              packageKey = packageName';
              namedDep = {
                ${packageKey} = packageLockEntry;
              };
            in
            if builtins.hasAttr packageKey foundSoFar then
              foundSoFar
            else if packageLockEntry ? dependencies then
              (builtins.foldl' go (namedDep // foundSoFar)) packageLockEntry.dependencies
            else
              foundSoFar // namedDep;
        in
        go { } packageName;

      cleanCargoLock = packages: {
        version = 4;
        package = lib.unique (
          lib.flatten (map (x: builtins.attrValues (getAllPackageDependencies x)) packages)
        );
      };

      # gets all the local (i.e. path) dependencies for a crate, recursively.
      #
      # note that to make this easier, we define all local dependencies as workspace dependencies.
      #
      # sig :: [string] -> bool -> [string]
      getMemberDeps =
        dirs: dontRemoveDevDeps:
        let
          go =
            dir': foundSoFar:
            let
              dirCargoToml = crateCargoToml dir';
            in
            lib.pipe
              (
                dirCargoToml.dependencies
                // (lib.optionalAttrs dontRemoveDevDeps dirCargoToml.dev-dependencies or { })
                // dirCargoToml.build-dependencies or { }
              )
              [
                (lib.filterAttrs (
                  dependency: value:
                  # ...and dep is a workspace dependency...
                  (value.workspace or false)
                  # ...and that workspace dependency is a path dependency...
                  && (builtins.hasAttr "path" workspaceCargoToml.workspace.dependencies.${dependency})
                  # ...and that workspace dependency has not been found yet (to prevent infinite recursion)
                  && !(builtins.elem workspaceCargoToml.workspace.dependencies.${dependency}.path foundSoFar)
                ))
                (lib.mapAttrsToList (
                  name: _value:
                  let
                    inherit (workspaceCargoToml.workspace.dependencies.${name}) path;
                  in
                  (go path (lib.unique (foundSoFar ++ [ path ]))) ++ [ path ]
                ))
                (lib.concat [ dir' ])
                lib.flatten
                lib.unique
              ];
        in
        lib.unique (lib.flatten (builtins.map (dir: go dir [ ]) dirs));

      # gets all the dependencies for a crate, recursively.
      #
      # sig :: [string] -> bool -> [string]
      getAllDeps =
        dirs: dontRemoveDevDeps:
        lib.pipe (getMemberDeps dirs dontRemoveDevDeps) [
          (map (
            path:
            ((crateCargoToml path).dependencies or { })
            // (lib.optionalAttrs dontRemoveDevDeps (crateCargoToml path).dev-dependencies or { })
            // ((crateCargoToml path).build-dependencies or { })
          ))
          (builtins.foldl' lib.recursiveUpdate { })
        ];

      # sig :: string -> attrs -> drv
      buildWorkspaceMember =
        # the directory that contains the Cargo.toml and src/ for the crate,
        # relative to the repository root, or a list of multiple crates.
        crateDirFromRoot:
        {
          # a suffix to add to the package name.
          pnameSuffix ? "",

          # the pname to use for this derivation if building multiple packages.
          pname ? null,
          # the version to use for this derivation if building multiple packages.
          version ? null,

          # extra args to be passed to cargo build.
          cargoBuildExtraArgs ? "",

          # if set to a string, the crate will be built for the specified target and will
          # rebuild the std library. incompatible with `cargoBuildRustToolchain`.
          buildStdTarget ? null,
          # update the toolchain that will be used for cargo build. defaults to
          # rust.toolchains.nightly if not set. incompatible with `buildStdTarget`.
          cargoBuildRustToolchain ? null,

          # rustflags to be passed to cargo build.
          rustflags ? "",

          # checkPhase to be passed to the cargo build derivation.
          cargoBuildCheckPhase ? null,
          # installPhase to be passed to the cargo build derivation.
          cargoBuildInstallPhase ? null,

          # standard postBuild phase.
          postBuild ? null,
          # standard postInstall phase.
          postInstall ? null,

          # extra environment variables to pass to the derivation.
          extraEnv ? { },
          # extra environment variables to pass to the derivation, only for crane.buildPackage.
          extraBuildEnv ? { },

          extraBuildInputs ? [ ],
          extraNativeBuildInputs ? [ ],

          # this builder will by default remove dev-dependencies from the Cargo.toml of all crates in the filtered source of the packages being built. set this to true to disable this behaviour.
          dontRemoveDevDeps ? false,

          # the root Cargo.toml may require patching when building certain packages in the monorepo. this hook can be used to arbitrarily modify the patched Cargo.toml before writing it into the source root derivation.
          rootCargoTomlHook ? x: x,
        }:
        assert builtins.isAttrs extraEnv;
        assert lib.assertMsg
          (
            (buildStdTarget != null -> cargoBuildRustToolchain == null)
            && (cargoBuildRustToolchain != null -> buildStdTarget == null)
          )
          "cannot set both buildStdTarget (${toString buildStdTarget}) and cargoBuildRustToolchain (${toString cargoBuildRustToolchain})";
        let
          pnameSuffix' = pnameSuffix;

          # normalize the crate info passed in, such that we can support both single and multiple packages with the same attribute
          processedCrateInfo =
            if (builtins.isList crateDirFromRoot) then
              assert isListOf builtins.isString crateDirFromRoot;
              {
                crateDirsFromRoot' = crateDirFromRoot;
                pname' =
                  assert builtins.isString pname;
                  pname;
                version' =
                  assert builtins.isString version;
                  version;
              }
            else if (builtins.isString crateDirFromRoot) then
              let
                cargoToml = (crateCargoToml crateDirFromRoot).package;
              in
              {
                crateDirsFromRoot' = [ crateDirFromRoot ];
                version' = cargoToml.version;
                pname' = cargoToml.name;
              }
            else
              abort "expected crateDirFromRoot to be a string or a list of strings, but it was a ${builtins.typeOf crateDirFromRoot}: ${toString crateDirFromRoot}";

          inherit (processedCrateInfo)
            crateDirsFromRoot'
            pname'
            version'
            ;

          # the rust toolchain that will be used to build the crate.
          # if build-std, use either the provided target or the default nightly toolchain. otherwise, just use the passed in toolchain.
          # the assertions at the beginning of this function ensure that these branches are exhaustive.
          cargoBuildRustToolchain' =
            if (cargoBuildRustToolchain == null) then
              (
                if buildStdTarget == null then
                  rust.toolchains.nightly
                else
                  rust.mkBuildStdToolchain { targets = [ buildStdTarget ]; }
              )
            else
              cargoBuildRustToolchain;

          cargoBuild = craneLib.overrideToolchain cargoBuildRustToolchain';

          memberDepsForCrate = getMemberDeps crateDirsFromRoot' dontRemoveDevDeps;
          memberDepsForCrateCargoTomls = readMemberCargoTomls memberDepsForCrate;

          patchedCargoLock = cleanCargoLock (map (dir: (crateCargoToml dir).package.name) crateDirsFromRoot');

          allDepsForCrate = getAllDeps memberDepsForCrate dontRemoveDevDeps;

          patchedCargoToml = {
            workspace = workspaceCargoToml.workspace // {
              members = memberDepsForCrate;
              dependencies = lib.filterAttrs (
                dep: _: builtins.hasAttr dep allDepsForCrate
              ) workspaceCargoToml.workspace.dependencies;
            };
            # only patch dependencies this crate actually depends on, since anything not in the lockfile will not be vendored by crane
            patch = workspaceCargoToml.patch // {
              crates-io = lib.filterAttrs (
                depName: _patch: builtins.any (lockPackage: lockPackage.name == depName) patchedCargoLock.package
              ) workspaceCargoToml.patch.crates-io;
            };
          };

          # patch the workspace Cargo.toml and Cargo.lock to only contain the local dependencies required to build this crate
          crateRepoSource = mkCleanSrc {
            inherit dontRemoveDevDeps;
            workspaceMembers = memberDepsForCrate;
            extraIncludes =
              (getIncludes memberDepsForCrateCargoTomls) ++ (getExtraIncludes memberDepsForCrateCargoTomls);
            cargoToml = writeTOML "Cargo.toml" (rootCargoTomlHook patchedCargoToml);
            cargoLock = writeTOML "Cargo.lock" patchedCargoLock;
          };

          # build the package.
          #
          # sig :: bool -> attrs
          builder =
            release:
            let
              packageFilterArgs = lib.concatMapStringsSep " " (
                dir: "-p ${(crateCargoToml dir).package.name}"
              ) crateDirsFromRoot';

              crateAttrs = extraEnv // {
                pname = pname';
                version = version';

                dummySrc = craneLib.mkDummySrc {
                  src = crateRepoSource;
                };

                # defaults to "--all-targets" otherwise, which breaks some stuff
                cargoCheckExtraArgs = "";

                buildInputs =
                  [
                    pkgs.pkg-config
                    pkgs.openssl
                  ]
                  ++ (lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ])
                  ++ extraBuildInputs;

                # [ pkgs.breakpointHook ] ++
                nativeBuildInputs = extraNativeBuildInputs;

                cargoVendorDir = craneLib.vendorMultipleCargoDeps {
                  inherit (craneLib.findCargoFiles crateRepoSource) cargoConfigs;
                  cargoLockList = lib.optionals (buildStdTarget != null) [
                    ./rust-std-Cargo.lock
                  ];
                  cargoLockParsedList = [
                    patchedCargoLock
                  ];
                };

                PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

                # RUST_MIN_STACK = 16777216; # ICE fix: maybe related to https://github.com/rust-lang/rust/issues/131419

                # we don't want to run cargo check or cargo test on this derivation since we do that in a separate package
                doCheck = false;

                pnameSuffix = pnameSuffix' + (lib.optionalString release "-release");

                cargoExtraArgs =
                  # REVIEW: Can -j1 only be specified for buildPackage and still get deterministic builds?
                  "${lib.optionalString release "-j1"} ${packageFilterArgs} ${cargoBuildExtraArgs}"
                  + (lib.optionalString (buildStdTarget != null)
                    # the leading space is important here!
                    " -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target ${buildStdTarget}"
                  );
                RUSTFLAGS = rustflags;

                preBuild =
                  (lib.concatMapStringsSep "\n\n" (dir: ''
                    if test -f ${dir}/src/main.rs; then
                      echo "extern crate embed_commit as _;" >> ${dir}/src/main.rs
                    else
                      echo "extern crate embed_commit as _;" >> ${dir}/src/lib.rs
                    fi
                  '') crateDirsFromRoot')
                  + lib.optionalString release ''
                    echo "cargoVendorDir: ${crateAttrs.cargoVendorDir}"
                    echo "rustToolchain: ${cargoBuildRustToolchain'}"

                    # find ${crateAttrs.cargoVendorDir} -maxdepth 1 -xtype d | grep -v '^${crateAttrs.cargoVendorDir}$' | sed -E 's@(.+)@ --remap-path-prefix=\1=/@g'

                    export RUSTFLAGS="$RUSTFLAGS $(find ${crateAttrs.cargoVendorDir} -maxdepth 1 -xtype d | grep -v '^${crateAttrs.cargoVendorDir}$' | sed -E 's@(.+)@ --remap-path-prefix=\1=@g' | tr '\n' ' ')  --remap-path-prefix=${cargoBuildRustToolchain'}/lib/rustlib/src/rust/library/alloc/src/= --remap-path-prefix=${cargoBuildRustToolchain'}/lib/rustlib/src/rust/library/std/src/= --remap-path-prefix=${cargoBuildRustToolchain'}/lib/rustlib/src/rust/library/core/src/="

                    echo "$RUSTFLAGS"
                  '';
              };

              cargoArtifacts = cargoBuild.buildDepsOnly crateAttrs;
            in

            (cargoBuild.buildPackage (
              extraBuildEnv
              // crateAttrs
              // {
                src = crateRepoSource;
              }
              // (lib.optionalAttrs (builtins.length crateDirsFromRoot' == 1) {
                meta.mainProgram = pname';
              })
              // (lib.optionalAttrs (cargoBuildInstallPhase != null) {
                installPhaseCommand = cargoBuildInstallPhase;
              })
              // (lib.optionalAttrs (postBuild != null) {
                inherit postBuild;
              })
              // (lib.optionalAttrs (postInstall != null) {
                inherit postInstall;
              })
              // (lib.optionalAttrs (cargoBuildCheckPhase != null) {
                checkPhase = cargoBuildCheckPhase;
              })
              // {
                inherit cargoArtifacts;
              }
              # for release builds, embed the git rev
              // (lib.optionalAttrs release (
                assert lib.assertMsg
                  (builtins.any (x: x) (
                    lib.mapAttrsToList (_: toml: toml.package.name == "embed-commit") memberDepsForCrateCargoTomls
                  ))
                  "crate ${pname'} does not depend on `embed-commit`, which is required for versioned release builds.";
                {
                  GIT_REV = gitRev;
                }
              ))
            )).overrideAttrs
              (
                old:
                {
                  passthru = (old.passthru or { }) // {
                    inherit release;
                    craneAttrs = crateAttrs // {
                      src = crateRepoSource;
                      inherit cargoArtifacts;
                    };
                  };
                }
                // old
              );
        in
        {
          "${pname'}${pnameSuffix'}" = (builder false) // {
            release = builder true;
          };
        };

      allCargoTomls = builtins.listToAttrs (
        map (
          dep: lib.nameValuePair dep (lib.importTOML "${root}/${dep}/Cargo.toml")
        ) workspaceCargoToml.workspace.members
      );

      cargoWorkspaceSrc = mkCleanSrc {
        workspaceMembers = workspaceCargoToml.workspace.members;
        extraIncludes = (getIncludes allCargoTomls) ++ (getExtraIncludes allCargoTomls);
        cargoToml = ../../Cargo.toml;
        cargoLock = ../../Cargo.lock;
        dontRemoveDevDeps = true;
      };
    in
    {
      _module.args = {
        crane =
          {
            lib = craneLib;
            inherit buildWorkspaceMember cargoWorkspaceSrc;
          }
          // (import ./buildWasmContract.nix {
            inherit
              buildWorkspaceMember
              crateCargoToml
              pkgs
              lib
              rust
              craneLib
              dbg
              gitRev
              ;
          });
      };

      checks =
        let
          cargoWorkspaceAttrs = {
            pname = "cargo-workspace";
            version = "0.0.0";
            src = cargoWorkspaceSrc;

            # unionvisor is tested individually, and mpc* crates attempt to link to galoisd (and don't have any tests anyways).
            cargoTestExtraArgs = "--workspace --exclude 'mpc*' --exclude unionvisor --no-fail-fast";
            cargoClippyExtraArgs = "--workspace --tests -- -Dwarnings";

            CARGO_PROFILE = "dev";
            SQLX_OFFLINE = true;
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
            ICS23_TEST_SUITE_DATA_DIR = "${inputs.ics23}/testdata";

            buildInputs = [
              pkgs.pkg-config
              pkgs.openssl
              pkgs.protobuf
              pkgs.perl
              pkgs.gnumake
              pkgs.systemd
            ] ++ (lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]);
            nativeBuildInputs = [
              pkgs.clang
            ];
          };
          cargoArtifacts = craneLib.buildDepsOnly cargoWorkspaceAttrs;
        in
        {
          cargo-workspace-clippy = craneLib.cargoClippy (cargoWorkspaceAttrs // { inherit cargoArtifacts; });
          cargo-workspace-test = craneLib.cargoTest (cargoWorkspaceAttrs // { inherit cargoArtifacts; });
          # NOTE: This is currently broken, as some crate features are not working properly
          all-crates-buildable-individually = craneLib.mkCargoDerivation (
            (builtins.removeAttrs cargoWorkspaceAttrs [
              "cargoTestExtraArgs"
              "cargoClippyExtraArgs"
            ])
            // {
              inherit cargoArtifacts;
              pname = "cargo-workspace-individual-check";
              # strictDeps = true;
              passAsFile = [ "actualBuildPhase" ];
              buildPhaseCargoCommand = null;
              buildPhase = ''
                . "$actualBuildPhasePath"
              '';
              # if we don't do this (and pass as file above), we hit "Argument list too long"
              # no clue why
              actualBuildPhase = lib.concatMapStringsSep "\n\n" (
                cargoToml:
                let
                  features = builtins.attrNames (builtins.removeAttrs (cargoToml.features or { }) [ "default" ]);

                  subsets =
                    xs: n:
                    if n == 0 then
                      [ [ ] ]
                    else if xs == [ ] then
                      [ ]
                    else
                      let
                        x = builtins.head xs;
                        xs' = builtins.tail xs;
                      in
                      (map (ys: [ x ] ++ ys) (subsets xs' (n - 1))) ++ (subsets xs' n);

                  allFeatureCombinations = lib.concatLists (
                    builtins.genList (subsets features) (dbg (builtins.length (dbg features) + 1))
                  );
                in
                if cargoToml.package.name == "protos" then
                  "cargo clippy -p protos --all-features --tests -- -Dwarnings"
                else if features == [ ] then
                  "cargo clippy -p ${cargoToml.package.name} --no-default-features --tests -- -Dwarnings"
                else
                  lib.concatMapStringsSep "\n" (
                    features:
                    "cargo clippy -p ${cargoToml.package.name} --no-default-features ${
                      lib.optionalString (features != [ ]) "-F${lib.concatMapStringsSep "," (f: f) features}"
                    } --tests -- -Dwarnings"
                  ) allFeatureCombinations
              ) (builtins.attrValues allCargoTomls);
              doInstallCargoArtifacts = false;
            }
          );
          # this would probably be better for caching but it's an insanely massive derivation
          # all-crates-buildable-individually = pkgs.linkFarmFromDrvs "all-crates-buildable-individually" (
          #   dbg (
          #     map (
          #       p:
          #       craneLib.cargoClippy (
          #         cargoWorkspaceAttrs
          #         // {
          #           inherit cargoArtifacts;
          #           pname = "${p.name}-individual-check";
          #           cargoClippyExtraArgs = "-p ${p.name} --tests -- -Dwarnings";
          #           doInstallCargoArtifacts = false;
          #         }
          #       )
          #     ) (pkgs.lib.mapAttrsToList lib.nameValuePair allCargoTomls)
          #   )
          # );
        };

      # these are incredibly useful for debugging
      packages = {
        cleanCargoLock = writeTOML "Cargo.lock" (cleanCargoLock [ "ibc-union" ]);
        # cleanCargoLock = writeTOML "Cargo.lock" (
        #   cleanCargoLock (
        #     builtins.attrNames (
        #       ((crateCargoToml "cosmwasm/ibc-union/core").dependencies or { })
        #       // ((crateCargoToml "cosmwasm/ibc-union/core").build-dependencies or { })
        #       // (lib.optionalAttrs false (crateCargoToml "cosmwasm/ibc-union/core").dev-dependencies or { })
        #     )
        #   )
        # );
        # getAllDeps = dbg (getAllDeps [ "cosmwasm/ibc-union/core" ]);
        # getDependency = dbg (
        #   getCargoLockPackageEntry "static_assertions 1.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
        # );
        # normalizedCargoLock = writeJSON "normalized-Cargo.lock.json" normalizedCargoLock;
        check-all-workspace-members-individually = pkgs.writeShellApplication {
          name = "check-all-workspace-members-individually";
          text = ''
            cargo metadata --no-deps | jq '.workspace_members[]' -r | xargs -I{} cargo check -p {}
          '';
        };
      };

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

{
  lib,
  crane,
  rustToolchain,
  fetchpatch,
  clang,
  cmake,
  copyDesktopItems,
  curl,
  perl,
  pkg-config,
  protobuf,
  fontconfig,
  freetype,
  libgit2,
  openssl,
  sqlite,
  zlib,
  zstd,
  alsa-lib,
  libxkbcommon,
  wayland,
  libglvnd,
  xorg,
  stdenv,
  makeFontsConf,
  vulkan-loader,
  envsubst,
  cargo-about,
  cargo-bundle,
  git,
  apple-sdk_15,
  darwinMinVersionHook,
  makeWrapper,
  nodejs_22,
  nix-gitignore,

  withGLES ? false,
}:

assert withGLES -> stdenv.hostPlatform.isLinux;

let
  includeFilter =
    path: type:
    let
      baseName = baseNameOf (toString path);
      parentDir = dirOf path;
      inRootDir = type == "directory" && parentDir == ../.;
    in
    !(
      inRootDir
      && (baseName == "docs" || baseName == ".github" || baseName == ".git" || baseName == "target")
    );
  craneLib = crane.overrideToolchain rustToolchain;
  commonSrc = lib.cleanSourceWith {
    src = nix-gitignore.gitignoreSource [ ] ../.;
    filter = includeFilter;
    name = "source";
  };
  commonArgs = rec {
    pname = "editsync-editor";
    version = "nightly";

    src = commonSrc;

    nativeBuildInputs =
      [
        clang
        cmake
        copyDesktopItems
        curl
        perl
        pkg-config
        protobuf
        cargo-about
      ]
      ++ lib.optionals stdenv.hostPlatform.isLinux [ makeWrapper ]
      ++ lib.optionals stdenv.hostPlatform.isDarwin [ cargo-bundle ];

    buildInputs =
      [
        curl
        fontconfig
        freetype
        libgit2
        openssl
        sqlite
        zlib
        zstd
      ]
      ++ lib.optionals stdenv.hostPlatform.isLinux [
        alsa-lib
        libxkbcommon
        wayland
        xorg.libxcb
      ]
      ++ lib.optionals stdenv.hostPlatform.isDarwin [
        apple-sdk_15
        (darwinMinVersionHook "10.15")
      ];

    env = {
      ZSTD_SYS_USE_PKG_CONFIG = true;
      FONTCONFIG_FILE = makeFontsConf {
        fontDirectories = [
          "${src}/assets/fonts/plex-mono"
          "${src}/assets/fonts/plex-sans"
        ];
      };
      EDITSYNC_UPDATE_EXPLANATION = "Editsync has been installed using Nix. Auto-updates have thus been disabled.";
      RELEASE_VERSION = version;
    };
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // rec {
    inherit cargoArtifacts;

    patches =
      [
        # Editsync uses cargo-install to install cargo-about during the script execution.
        # We provide cargo-about ourselves and can skip this step.
        # Until https://github.com/khulnasoft/editsync/issues/19971 is fixed,
        # we also skip any crate for which the license cannot be determined.
        (fetchpatch {
          url = "https://raw.githubusercontent.com/NixOS/nixpkgs/1fd02d90c6c097f91349df35da62d36c19359ba7/pkgs/by-name/ze/editsync-editor/0001-generate-licenses.patch";
          hash = "sha256-cLgqLDXW1JtQ2OQFLd5UolAjfy7bMoTw40lEx2jA2pk=";
        })
      ]
      ++ lib.optionals stdenv.hostPlatform.isDarwin [
        # Livekit requires Swift 6
        # We need this until livekit-rust sdk is used
        (fetchpatch {
          url = "https://raw.githubusercontent.com/NixOS/nixpkgs/1fd02d90c6c097f91349df35da62d36c19359ba7/pkgs/by-name/ze/editsync-editor/0002-disable-livekit-darwin.patch";
          hash = "sha256-whZ7RaXv8hrVzWAveU3qiBnZSrvGNEHTuyNhxgMIo5w=";
        })
      ];

    cargoExtraArgs = "--package=editsync --package=cli --features=gpui/runtime_shaders";

    dontUseCmakeConfigure = true;
    preBuild = ''
      bash script/generate-licenses
    '';

    postFixup = lib.optionalString stdenv.hostPlatform.isLinux ''
      patchelf --add-rpath ${gpu-lib}/lib $out/libexec/*
      patchelf --add-rpath ${wayland}/lib $out/libexec/*
      wrapProgram $out/libexec/editsync-editor --suffix PATH : ${lib.makeBinPath [ nodejs_22 ]}
    '';

    RUSTFLAGS = if withGLES then "--cfg gles" else "";
    gpu-lib = if withGLES then libglvnd else vulkan-loader;

    preCheck = ''
      export HOME=$(mktemp -d);
    '';

    cargoTestExtraArgs =
      "-- "
      + lib.concatStringsSep " " (
        [
          # Flaky: unreliably fails on certain hosts (including Hydra)
          "--skip=editsync::tests::test_window_edit_state_restoring_enabled"
        ]
        ++ lib.optionals stdenv.hostPlatform.isLinux [
          # Fails on certain hosts (including Hydra) for unclear reason
          "--skip=test_open_paths_action"
        ]
      );

    installPhase =
      if stdenv.hostPlatform.isDarwin then
        ''
          runHook preInstall

          # cargo-bundle expects the binary in target/release
          mv target/release/editsync target/release/editsync

          pushd crates/editsync

          # Note that this is GNU sed, while Editsync's bundle-mac uses BSD sed
          sed -i "s/package.metadata.bundle-stable/package.metadata.bundle/" Cargo.toml
          export CARGO_BUNDLE_SKIP_BUILD=true
          app_path=$(cargo bundle --release | xargs)

          # We're not using the fork of cargo-bundle, so we must manually append plist extensions
          # Remove closing tags from Info.plist (last two lines)
          head -n -2 $app_path/Contents/Info.plist > Info.plist
          # Append extensions
          cat resources/info/*.plist >> Info.plist
          # Add closing tags
          printf "</dict>\n</plist>\n" >> Info.plist
          mv Info.plist $app_path/Contents/Info.plist

          popd

          mkdir -p $out/Applications $out/bin
          # Editsync expects git next to its own binary
          ln -s ${git}/bin/git $app_path/Contents/MacOS/git
          mv target/release/cli $app_path/Contents/MacOS/cli
          mv $app_path $out/Applications/

          # Physical location of the CLI must be inside the app bundle as this is used
          # to determine which app to start
          ln -s $out/Applications/Editsync.app/Contents/MacOS/cli $out/bin/editsync

          runHook postInstall
        ''
      else
        ''
          runHook preInstall

          mkdir -p $out/bin $out/libexec
          cp target/release/editsync $out/libexec/editsync-editor
          cp target/release/cli $out/bin/editsync

          install -D ${commonSrc}/crates/editsync/resources/app-icon@2x.png $out/share/icons/hicolor/1024x1024@2x/apps/editsync.png
          install -D ${commonSrc}/crates/editsync/resources/app-icon.png $out/share/icons/hicolor/512x512/apps/editsync.png

          # extracted from https://github.com/khulnasoft/editsync/blob/v0.141.2/script/bundle-linux (envsubst)
          # and https://github.com/khulnasoft/editsync/blob/v0.141.2/script/install.sh (final desktop file name)
          (
            export DO_STARTUP_NOTIFY="true"
            export APP_CLI="editsync"
            export APP_ICON="editsync"
            export APP_NAME="Editsync"
            export APP_ARGS="%U"
            mkdir -p "$out/share/applications"
            ${lib.getExe envsubst} < "crates/editsync/resources/editsync.desktop.in" > "$out/share/applications/dev.editsync.Editsync.desktop"
          )

          runHook postInstall
        '';

    meta = {
      description = "High-performance, multiplayer code editor from the creators of Atom and Tree-sitter";
      homepage = "https://editsync.khulnasoft.com";
      changelog = "https://editsync.khulnasoft.com/releases/preview";
      license = lib.licenses.gpl3Only;
      mainProgram = "editsync";
      platforms = lib.platforms.linux ++ lib.platforms.darwin;
    };
  }
)

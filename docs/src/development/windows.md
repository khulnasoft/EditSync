# Building Editsync for Windows

> The following commands may be executed in any shell.

## Repository

Clone down the [Editsync repository](https://github.com/khulnasoft/editsync).

## Dependencies

- Install [rustup](https://www.rust-lang.org/tools/install)

- Install [Visual Studio](https://visualstudio.microsoft.com/downloads/) with the optional components `MSVC v*** - VS YYYY C++ x64/x86 build tools` and `MSVC v*** - VS YYYY C++ x64/x86 Spectre-mitigated libs (latest)` (`v***` is your VS version and `YYYY` is year when your VS was released. Pay attention to the architecture and change it to yours if needed.)
- Install Windows 11 or 10 SDK depending on your system, but ensure that at least `Windows 10 SDK version 2104 (10.0.20348.0)` is installed on your machine. You can download it from the [Windows SDK Archive](https://developer.microsoft.com/windows/downloads/windows-sdk/)
- Install [CMake](https://cmake.org/download) (required by [a dependency](https://docs.rs/wasmtime-c-api-impl/latest/wasmtime_c_api/))

## Backend dependencies

> This section is still in development. The instructions are not yet complete.

If you are developing collaborative features of Editsync, you'll need to install the dependencies of editsync's `collab` server:

- Install [Postgres](https://www.postgresql.org/download/windows/)
- Install [Livekit](https://github.com/livekit/livekit-cli) and [Foreman](https://theforeman.org/manuals/3.9/quickstart_guide.html)

Alternatively, if you have [Docker](https://www.docker.com/) installed you can bring up all the `collab` dependencies using Docker Compose:

```sh
docker compose up -d
```

## Building from source

Once you have the dependencies installed, you can build Editsync using [Cargo](https://doc.rust-lang.org/cargo/).

For a debug build:

```sh
cargo run
```

For a release build:

```sh
cargo run --release
```

And to run the tests:

```sh
cargo test --workspace
```

## Installing from msys2

[MSYS2](https://msys2.org/) distribution provides Editsync as a package [mingw-w64-editsync](https://packages.msys2.org/base/mingw-w64-editsync). The package is available for UCRT64, MINGW64 and CLANG64 repositories. To download it, run

```sh
pacman -Syu
pacman -S $MINGW_PACKAGE_PREFIX-editsync
```

then you can run `editsync` in a shell.

You can see the [build script](https://github.com/msys2/MINGW-packages/blob/master/mingw-w64-editsync/PKGBUILD) for more details on build process.

> Please, report any issue in [msys2/MINGW-packages/issues](https://github.com/msys2/MINGW-packages/issues?q=is%3Aissue+is%3Aopen+editsync) first.

## Troubleshooting

### Can't compile editsync

Before reporting the issue, make sure that you have the latest rustc version with `rustup update`.

### Cargo errors claiming that a dependency is using unstable features

Try `cargo clean` and `cargo build`.

### `STATUS_ACCESS_VIOLATION`

This error can happen if you are using the "rust-lld.exe" linker. Consider trying a different linker.

If you are using a global config, consider moving the Editsync repository to a nested directory and add a `.cargo/config.toml` with a custom linker config in the parent directory.

See this issue for more information [#12041](https://github.com/khulnasoft/editsync/issues/12041)

### Invalid RC path selected

Sometimes, depending on the security rules applied to your laptop, you may get the following error while compiling Editsync:

```
error: failed to run custom build command for `editsync(C:\Users\USER\src\editsync\crates\editsync)`

Caused by:
  process didn't exit successfully: `C:\Users\USER\src\editsync\target\debug\build\editsync-b24f1e9300107efc\build-script-build` (exit code: 1)
  --- stdout
  cargo:rerun-if-changed=../../.git/logs/HEAD
  cargo:rustc-env=EDITSYNC_COMMIT_SHA=25e2e9c6727ba9b77415588cfa11fd969612adb7
  cargo:rustc-link-arg=/stack:8388608
  cargo:rerun-if-changed=resources/windows/app-icon.ico
  package.metadata.winresource does not exist
  Selected RC path: 'bin\x64\rc.exe'

  --- stderr
  The system cannot find the path specified. (os error 3)
warning: build failed, waiting for other jobs to finish...
```

In order to fix this issue, you can manually set the `EDITSYNC_RC_TOOLKIT_PATH` environment variable to the RC toolkit path. Usually, you can set it to:
`C:\Program Files (x86)\Windows Kits\10\bin\<SDK_version>\x64`.

See this [issue](https://github.com/khulnasoft/editsync/issues/18393) for more information.

### Build fails: Path too long

You may receive an error like the following when building

```
error: failed to get `pet` as a dependency of package `languages v0.1.0 (D:\a\editsync-windows-builds\editsync-windows-builds\crates\languages)`

Caused by:
  failed to load source for dependency `pet`

Caused by:
  Unable to update https://github.com/microsoft/python-environment-tools.git?rev=ffcbf3f28c46633abd5448a52b1f396c322e0d6c#ffcbf3f2

Caused by:
  path too long: 'C:/Users/runneradmin/.cargo/git/checkouts/python-environment-tools-903993894b37a7d2/ffcbf3f/crates/pet-conda/tests/unix/conda_env_without_manager_but_found_in_history/some_other_location/conda_install/conda-meta/python-fastjsonschema-2.16.2-py310hca03da5_0.json'; class=Filesystem (30)
```

In order to solve this, you can enable longpath support for git and Windows.

For git: `git config --system core.longpaths true`

And for Windows with this PS command:

```powershell
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

For more information on this, please see [win32 docs](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation?tabs=powershell)

(note that you will need to restart your system after enabling longpath support)

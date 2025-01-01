# Editsync on Linux

For most people we recommend using the script on the [download](https://editsync.khulnasoft.com/download) page to install Editsync:

```sh
curl -f https://editsync.khulnasoft.com/install.sh | sh
```

We also offer a preview build of Editsync which receives updates about a week ahead of stable. You can install it with:

```sh
curl -f https://editsync.khulnasoft.com/install.sh | EDITSYNC_CHANNEL=preview sh
```

The Editsync installed by the script works best on systems that:

- have a Vulkan compatible GPU available (for example Linux on an M-series macBook)
- have a system-wide glibc (NixOS and Alpine do not by default)
  - x86_64 (Intel/AMD): glibc version >= 2.31 (Ubuntu 20 and newer)
  - aarch64 (ARM): glibc version >= 2.35 (Ubuntu 22 and newer)

Both Nix and Alpine have third-party Editsync packages available (though they are currently a few weeks out of date). If you'd like to use our builds they do work if you install a glibc compatibility layer. On NixOS you can try [nix-ld](https://github.com/Mic92/nix-ld), and on Alpine [gcompat](https://wiki.alpinelinux.org/wiki/Running_glibc_programs).

You will need to build from source for:

- architectures other than 64-bit Intel or 64-bit ARM (for example a 32-bit or RISC-V machine)
- Redhat Enterprise Linux 8.x, Rocky Linux 8, AlmaLinux 8, Amazon Linux 2 on all architectures
- Redhat Enterprise Linux 9.x, Rocky Linux 9.3, AlmaLinux 8, Amazon Linux 2023 on aarch64 (x86_x64 OK)

## Other ways to install Editsync on Linux

Editsync is open source, and [you can install from source](./development/linux.md).

### Installing via a package manager

There are several third-party Editsync packages for various Linux distributions and package managers, sometimes under `editsync-editor`. You may be able to install Editsync using these packages:

- Flathub: [`dev.editsync.Editsync`](https://flathub.org/apps/dev.editsync.Editsync)
- Arch: [`editsync`](https://archlinux.org/packages/extra/x86_64/editsync/)
- Arch (AUR): [`editsync-git`](https://aur.archlinux.org/packages/editsync-git), [`editsync-preview`](https://aur.archlinux.org/packages/editsync-preview), [`editsync-preview-bin`](https://aur.archlinux.org/packages/editsync-preview-bin)
- Alpine: `editsync` ([aarch64](https://pkgs.alpinelinux.org/package/edge/testing/aarch64/editsync)) ([x86_64](https://pkgs.alpinelinux.org/package/edge/testing/x86_64/editsync))
- Nix: `editsync-editor` ([unstable](https://search.nixos.org/packages?channel=unstable&show=editsync-editor))
- Fedora/Ultramarine (Terra): [`editsync`](https://github.com/terrapkg/packages/tree/frawhide/anda/devs/editsync/stable), [`editsync-preview`](https://github.com/terrapkg/packages/tree/frawhide/anda/devs/editsync/preview), [`editsync-nightly`](https://github.com/terrapkg/packages/tree/frawhide/anda/devs/editsync/nightly)
- Solus: [`editsync`](https://github.com/getsolus/packages/tree/main/packages/z/editsync)
- Parabola: [`editsync`](https://www.parabola.nu/packages/extra/x86_64/editsync/)
- Manjaro: [`editsync`](https://packages.manjaro.org/?query=editsync)
- ALT Linux (Sisyphus): [`editsync`](https://packages.altlinux.org/en/sisyphus/srpms/editsync/)
- AOSC OS: [`editsync`](https://packages.aosc.io/packages/editsync)
- openSUSE Tumbleweed: [`editsync`](https://en.opensuse.org/Editsync)
- Please add others to this list!

When installing a third-party package please be aware that it may not be completely up to date and may be slightly different from the Editsync we package (a common change is to rename the binary to `editsyncit` or `editsyncitor` to avoid conflicting with other packages).

We'd love your help making Editsync available for everyone. If Editsync is not yet available for your package manager, and you would like to fix that, we have some notes on [how to do it](./development/linux.md#notes-for-packaging-editsync).

### Downloading manually

If you'd prefer, you can install Editsync by downloading our pre-built .tar.gz. This is the same artifact that our install script uses, but you can customize the location of your installation by modifying the instructions below:

Download the `.tar.gz` file:

- [editsync-linux-x86_64.tar.gz](https://editsync.khulnasoft.com/api/releases/stable/latest/editsync-linux-x86_64.tar.gz) ([preview](https://editsync.khulnasoft.com/api/releases/preview/latest/editsync-linux-x86_64.tar.gz))
- [editsync-linux-aarch64.tar.gz](https://editsync.khulnasoft.com/api/releases/stable/latest/editsync-linux-aarch64.tar.gz)
  ([preview](https://editsync.khulnasoft.com/api/releases/preview/latest/editsync-linux-aarch64.tar.gz))

Then ensure that the `editsync` binary in the tarball is on your path. The easiest way is to unpack the tarball and create a symlink:

```sh
mkdir -p ~/.local
# extract editsync to ~/.local/editsync.app/
tar -xvf <path/to/download>.tar.gz -C ~/.local
# link the editsync binary to ~/.local/bin (or another directory in your $PATH)
ln -sf ~/.local/editsync.app/bin/editsync ~/.local/bin/editsync
```

If you'd like integration with an XDG-compatible desktop environment, you will also need to install the `.desktop` file:

```sh
cp ~/.local/editsync.app/share/applications/editsync.desktop ~/.local/share/applications/dev.editsync.Editsync.desktop
sed -i "s|Icon=editsync|Icon=$HOME/.local/editsync.app/share/icons/hicolor/512x512/apps/editsync.png|g" ~/.local/share/applications/dev.editsync.Editsync.desktop
sed -i "s|Exec=editsync|Exec=$HOME/.local/editsync.app/libexec/editsync-editor|g" ~/.local/share/applications/dev.editsync.Editsync.desktop
```

## Troubleshooting

Linux works on a large variety of systems configured in many different ways. We primarily test Editsync on a vanilla Ubuntu setup, as it is the most common distribution our users use, that said we do expect it to work on a wide variety of machines.

### Editsync fails to start

If you see an error like "/lib64/libc.so.6: version 'GLIBC_2.29' not found" it means that your distribution's version of glibc is too old. You can either upgrade your system, or [install Editsync from source](./development/linux.md).

### Editsync fails to open windows

### Editsync is very slow

Editsync requires a GPU to run effectively. Under the hood, we use [Vulkan](https://www.vulkan.org/) to communicate with your GPU. If you are seeing problems with performance, or Editsync fails to load, it is possible that Vulkan is the culprit.

If you're using an AMD GPU, you might get a 'Broken Pipe' error. Try using the RADV or Mesa drivers. (See the following GitHub issue for more details: [#13880](https://github.com/khulnasoft/editsync/issues/13880)).

If you see a notification saying `Editsync failed to open a window: NoSupportedDeviceFound` this means that Vulkan cannot find a compatible GPU. You can begin troubleshooting Vulkan by installing the `vulkan-tools` package and running:

```sh
vkcube
```

This should output a line describing your current graphics setup and show a rotating cube. If this does not work, you should be able to fix it by installing Vulkan compatible GPU drivers, however in some cases (for example running Linux on an Arm-based MacBook) there is no Vulkan support yet.

If you see errors like `ERROR_INITIALIZATION_FAILED` or `GPU Crashed` or `ERROR_SURFACE_LOST_KHR` then you may be able to work around this by installing different drivers for your GPU, or by selecting a different GPU to run on. (See the following GitHub issue for more details: [#14225](https://github.com/khulnasoft/editsync/issues/14225))

As of Editsync v0.146.x we log the selected GPU driver and you should see `Using GPU: ...` in the Editsync log (`~/.local/share/editsync/logs/Editsync.log`).

If Editsync is selecting your integrated GPU instead of your discrete GPU, you can fix this by exporting the environment variable `DRI_PRIME=1` before running Editsync.

If you are using Mesa, and want more control over which GPU is selected you can run `MESA_VK_DEVICE_SELECT=list editsync --foreground` to get a list of available GPUs and then export `MESA_VK_DEVICE_SELECT=xxxx:yyyy` to choose a specific device.

If you are using `amdvlk` you may find that editsync only opens when run with `sudo $(which editsync)`. To fix this, remove the `amdvlk` and `lib32-amdvlk` packages and install mesa/vulkan instead. ([#14141](https://github.com/khulnasoft/editsync/issues/14141).

If you have a discrete GPU and you are using [PRIME](https://wiki.archlinux.org/title/PRIME) you may be able to configure Editsync to work by setting `/etc/prime-discrete` to 'on'.

For more information, the [Arch guide to Vulkan](https://wiki.archlinux.org/title/Vulkan) has some good steps that translate well to most distributions.

If Vulkan is configured correctly, and Editsync is still slow for you, please [file an issue](https://github.com/khulnasoft/editsync) with as much information as possible.

### I can't open any files

### Clicking links isn't working

These features are provided by XDG desktop portals, specifically:

- `org.freedesktop.portal.FileChooser`
- `org.freedesktop.portal.OpenURI`

Some window managers, such as `Hyprland`, don't provide a file picker by default. See [this list](https://wiki.archlinux.org/title/XDG_Desktop_Portal#List_of_backends_and_interfaces) as a starting point for alternatives.

### Editsync isn't remembering my API keys

### Editsync isn't remembering my login

These feature also requires XDG desktop portals, specifically:

- `org.freedesktop.portal.Secret` or
- `org.freedesktop.Secrets`

Editsync needs a place to securely store secrets such as your Editsync login cookie or your OpenAI API Keys and we use a system provided keychain to do this. Examples of packages that provide this are `gnome-keyring`, `KWallet` and `keepassxc` among others.

### Could not start inotify

Editsync relies on inotify to watch your filesystem for changes. If you cannot start inotify then Editsync will not work reliably.

If you are seeing "too many open files" then first try `sysctl fs.inotify`.

- You should see that max_user_instances is 128 or higher (you can change the limit with `sudo sysctl fs.inotify.max_user_instances=1024`). Editsync needs only 1 inotify instance.
- You should see that `max_user_watches` is 8000 or higher (you can change the limit with `sudo sysctl fs.inotify.max_user_watches=64000`). Editsync needs one watch per directory in all your open projects + one per git repository + a handful more for settings, themes, keymaps, extensions.

It is also possible that you are running out of file descriptors. You can check the limits with `ulimit` and update them by editing `/etc/security/limits.conf`.

### FIPS Mode OpenSSL internal error {#fips}

If your machine is running in FIPS mode (`cat /proc/sys/crypto/fips_enabled` is set to `1`) Editsync may fail to start and output the following when launched with `editsync --foreground`:

```
crypto/fips/fips.c:154: OpenSSL internal error: FATAL FIPS SELFTEST FAILURE
```

As a workaround, remove the bundled `libssl` and `libcrypto` libraries from the `editsync.app/lib` directory:

```
rm ~/.local/editsync.app/lib/libssl.so.1.1
rm ~/.local/editsync.app/lib/libcrypto.so.1.1
```

This will force editsync to fallback to the system `libssl` and `libcrypto` libraries.

### Editing files requiring root access

When you try to edit files that require root access, Editsync requires `pkexec` (part of polkit) to handle authentication prompts.

Polkit comes pre-installed with most desktop environments like GNOME and KDE. If you're using a minimal system and polkit is not installed, you can install it with:

- Ubuntu/Debian: `sudo apt install policykit-1`
- Fedora: `sudo dnf install polkit`
- Arch Linux: `sudo pacman -S polkit`

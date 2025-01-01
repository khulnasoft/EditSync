# Remote Development

Remote Development allows you to code at the speed of thought, even when your codebase is not on your local machine. You use Editsync locally so the UI is immediately responsive, but offload heavy computation to the development server so that you can work effectively.

> **Note:** Remoting is still "beta". We are still refining the reliability and performance.

## Overview

Remote development requires two computers, your local machine that runs the Editsync UI and the remote server which runs a Editsync headless server. The two communicate over SSH, so you will need to be able to SSH from your local machine into the remote server to use this feature.

![Architectural overview of Editsync Remote Development](https://editsync.khulnasoft.com/img/remote-development/diagram.png)

On your local machine, Editsync runs its UI, talks to language models, uses Tree-sitter to parse and syntax-highlight code, and store unsaved changes and recent projects. The source code, language servers, tasks, and the terminal all run on the remote server.

> **Note:** The original version of remote development sent traffic via Editsync's servers. As of Editsync v0.157 you can no-longer use that mode.

## Setup

1. Download and install the latest [Editsync](https://editsync.khulnasoft.com/releases). You need at least Editsync v0.159.
1. Open the remote projects dialogue with <kbd>cmd-shift-p remote</kbd> or <kbd>cmd-control-o</kbd>.
1. Click "Connect New Server" and enter the command you use to SSH into the server. See [Supported SSH options](#supported-ssh-options) for options you can pass.
1. Your local machine will attempt to connect to the remote server using the `ssh` binary on your path. Assuming the connection is successful, Editsync will download the server on the remote host and start it.
1. Once the Editsync server is running, you will be prompted to choose a path to open on the remote server.
   > **Note:** Editsync does not currently handle opening very large directories (for example, `/` or `~` that may have >100,000 files) very well. We are working on improving this, but suggest in the meantime opening only specific projects, or subfolders of very large mono-repos.

For simple cases where you don't need any SSH arguments, you can run `editsync ssh://[<user>@]<host>[:<port>]/<path>` to open a remote folder/file directly. If you'd like to hotlink into an SSH project, use a link of the format: `editsync://ssh/[<user>@]<host>[:<port>]/<path>`.

## Supported platforms

The remote machine must be able to run Editsync's server. The following platforms should work, though note that we have not exhaustively tested every Linux distribution:

- macOS Catalina or later (Intel or Apple Silicon)
- Linux (x86_64 or arm64, we do not yet support 32-bit platforms)
- Windows is not yet supported.

## Configuration

The list of remote servers is stored in your settings file {#kb editsync::OpenSettings}. You can edit this list using the Remote Projects dialogue {#kb projects::OpenRemote}, which provides some robustness - for example it checks that the connection can be established before writing it to the settings file.

```json
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "projects": ["~/code/editsync/editsync"]
    }
  ]
}
```

Editsync shells out to the `ssh` on your path, and so it will inherit any configuration you have in `~/.ssh/config` for the given host. That said, if you need to override anything you can configure the following additional options on each connection:

```json
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "projects": ["~/code/editsync/editsync"],
      // any argument to pass to the ssh master process
      "args": ["-i", "~/.ssh/work_id_file"],
      "port": 22, // defaults to 22
      // defaults to your username on your local machine
      "username": "me"
    }
  ]
}
```

There are two additional Editsync-specific options per connection, `upload_binary_over_ssh` and `nickname`:

```json
{
  "ssh_connections": [
    {
      "host": "192.168.1.10",
      "projects": ["~/code/editsync/editsync"],
      // by default Editsync will download the server binary from the internet on the remote.
      // When this is true, it'll be downloaded to your laptop and uploaded over SSH.
      // This is useful when your remote server has restricted internet access.
      "upload_binary_over_ssh": true,
      // Shown in the Editsync UI to help distinguish multiple hosts.
      "nickname": "lil-linux"
    }
  ]
}
```

If you use the command line to open a connection to a host by doing `editsync ssh://192.168.1.10/~/.vimrc`, then extra options are read from your settings file by finding the first connection that matches the host/username/port of the URL on the command line.

Additionally it's worth noting that while you can pass a password on the command line `editsync ssh://user:password@host/~`, we do not support writing a password to your settings file. If you're connecting repeatedly to the same host, you should configure key-based authentication.

## Editsync settings

When opening a remote project there are three relevant settings locations:

- The local Editsync settings (in `~/.editsync/settings.json` on macOS or `~/.config/editsync/settings.json` on Linux) on your local machine.
- The server Editsync settings (in the same place) on the remote server.
- The project settings (in `.editsync/settings.json` or `.editorconfig` of your project)

Both the local Editsync and the server Editsync read the project settings, but they are not aware of the other's main `settings.json`.

Depending on the kind of setting you want to make, which settings file you should use:

- Project settings should be used for things that affect the project: indentation settings, which formatter / language server to use, etc.
- Server settings should be used for things that affect the server: paths to language servers, etc.
- Local settings should be used for things that affect the UI: font size, etc.

## Initializing the remote server

Once you provide the SSH options, Editsync shells out to `ssh` on your local machine to create a ControlMaster connection with the options you provide.

Any prompts that SSH needs will be shown in the UI, so you can verify host keys, type key passwords, etc.

Once the master connection is established, Editsync will check to see if the remote server binary is present in `~/.editsync_server` on the remote, and that its version matches the current version of Editsync that you're using.

If it is not there or the version mismatches, Editsync will try to download the latest version. By default, it will download from `https://editsync.khulnasoft.com` directly, but if you set: `{"upload_binary_over_ssh":true}` in your settings for that server, it will download the binary to your local machine and then upload it to the remote server.

If you'd like to maintain the server binary yourself you can. You can either download our prebuilt versions from [Github](https://github.com/khulnasoft/editsync/releases), or [build your own](https://editsync.khulnasoft.com/docs/development) with `cargo build -p remote_server --release`. If you do this, you must upload it to `~/.editsync_server/editsync-remote-server-{RELEASE_CHANNEL}-{OS}-{ARCH}` on the server, for example `.editsync-server/editsync-remote-server-preview-linux-x86_64`. The version must exactly match the version of Editsync itself you are using.

## Maintaining the SSH connection

Once the server is initialieditsync. Editsync will create new SSH connections (reusing the existing ControlMaster) to run the remote development server.

Each connection tries to run the development server in proxy mode. This mode will start the daemon if it is not running, and reconnect to it if it is. This way when your connection drops and is restarted, you can continue to work without interruption.

In the case that reconnecting fails, the daemon will not be re-used. That said, unsaved changes are by default persisted locally, so that you do not lose work. You can always reconnect to the project at a later date and Editsync will restore unsaved changes.

If you are struggling with connection issues, you should be able to see more information in the Editsync log `cmd-shift-p Open Log`. If you are seeing things that are unexpected, please file a [GitHub issue](https://github.com/khulnasoft/editsync/issues/new) or reach out in the #remoting-feedback channel in the [Editsync Discord](https://editsync.khulnasoft.com/community-links).

## Supported SSH Options

Under the hood, Editsync shells out to the `ssh` binary to connect to the remote server. We create one SSH control master per project, and use then use that to multiplex SSH connections for the Editsync protocol itself, any terminals you open and tasks you run. We read settings from your SSH config file, but if you want to specify additional options to the SSH control master you can configure Editsync to set them.

When typing in the "Connect New Server" dialogue, you can use bash-style quoting to pass options containing a space. Once you have created a server it will be added to the `"ssh_connections": []` array in your settings file. You can edit the settings file directly to make changes to SSH connections.

Supported options:

- `-p` / `-l` - these are equivalent to passing the port and the username in the host string.
- `-L` / `-R` for port forwarding
- `-i` - to use a specific key file
- `-o` - to set custom options
- `-J` / `-w` - to proxy the SSH connection
- And also... `-4`, `-6`, `-A`, `-a`, `-C`, `-K`, `-k`, `-X`, `-x`, `-Y`, `-y`, `-B`, `-b`, `-c`, `-D`, `-I`, `-i`, `-J`, `-l`, `-m`, `-o`, `-P`, `-p`, `-w`

Note that we deliberately disallow some options (for example `-t` or `-T`) that Editsync will set for you.

## Known Limitations

- Editsync extensions are not yet supported on remotes, so languages that need them for support do not work.
- You can't open files from the remote Terminal by typing the `editsync` command.
- Editsync does not yet support automatic port-forwarding. You can use `-R` and `-L` in your SSH arguments for now.

## Feedback

Please join the #remoting-feedback channel in the [Editsync Discord](https://editsync.khulnasoft.com/community-links).
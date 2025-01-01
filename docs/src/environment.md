# Environment Variables

_**Note**: The following only applies to Editsync 0.152.0 and later._

Multiple features in Editsync are affected by environment variables:

- Tasks
- Built-in terminal
- Look-up of language servers
- Language servers

In order to make the best use of these features, it's helpful to understand where Editsync gets its environment variables from and how they're used.

## Where does Editsync get its environment variables from?

How Editsync was started — whether it's icon was clicked in the macOS Dock or in a Linux window manager, or whether it was started via the CLI `editsync` that comes with Editsync — influences which environment variables Editsync can use.

### Launched from the CLI

If Editsync is opened via the CLI (`editsync`), it will inherit the environment variables from the surrounding shell session.

That means if you do

```
$ export MY_ENV_VAR=hello
$ editsync .
```

the environment variable `MY_ENV_VAR` is now available inside Editsync. For example, in the built-in terminal.

Starting with Editsync 0.152.0, the CLI `editsync` will _always_ pass along its environment to Editsync, regardless of whether a Editsync instance was previously running or not. Prior to Editsync 0.152.0 this was not the case and only the first Editsync instance would inherit the environment variables.

### Launched via window manager, Dock, or launcher

When Editsync has been launched via the macOS Dock, or a GNOME or KDE icon on Linux, or an application launcher like Alfred or Raycast, it has no surrounding shell environment from which to inherit its environment variables.

In order to still have a useful environment, Editsync spawns a login shell in the user's home directory and gets its environment. This environment is then set on the Editsync _process_. That means all Editsync windows and projects will inherit that home directory environment.

Since that can lead to problems for users that require different environment variables for a project (because they use `direnv`, or `asdf`, or `mise`, ... in that project), when opening project, Editsync spawns another login shell. This time in the project's directory. The environment from that login shell is _not_ set on the process (because that would mean opening a new project changes the environment for all Editsync windows). Instead, the environment is stored and passed along when running tasks, opening terminals, or spawning language servers.

## Where and how are environment variables used?

There are two sets of environment variables:

1. Environment variables of the Editsync process
2. Environment variables stored per project

The variables from (1) are always used, since they are stored on the process itself and every spawned process (tasks, terminals, language servers, ...) will inherit them by default.

The variables from (2) are used explicitly, depending on the feature.

### Tasks

Tasks are spawned with an combined environment. In order of precedence (low to high, with the last overwriting the first):

- the Editsync process environment
- if the project was opened from the CLI: the CLI environment
- if the project was not opened from the CLI: the project environment variables obtained by running a login shell in the project's root folder
- optional, explicitly configured environment in settings

### Built-in terminal

Built-in terminals, like tasks, are spawned with an combined environment. In order of precedence (low to high):

- the Editsync process environment
- if the project was opened from the CLI: the CLI environment
- if the project was not opened from the CLI: the project environment variables obtained by running a login shell in the project's root folder
- optional, explicitly configured environment in settings

### Look-up of language servers

For some languages the language server adapters lookup the binary in the user's `$PATH`. Examples:

- Go
- Zig
- Rust (if [configured to do so](./languages/rust.md#binary))
- C
- TypeScript

For this look-up, Editsync uses the following the environment:

- if the project was opened from the CLI: the CLI environment
- if the project was not opened from the CLI: the project environment variables obtained by running a login shell in the project's root folder

### Language servers

After looking up a language server, Editsync starts them.

These language server processes always inherit Editsync's process environment. But, depending on the language server look-up, additional environment variables might be set or overwrite the process environment.

- If the language server was found in the project environment's `$PATH`, then the project environment's is passed along to the language server process. Where the project environment comes from depends on how the project was opened, via CLI or not. See previous point on look-up of language servers.
- If the language servers was not found in the project environment, Editsync tries to install it globally and start it globally. In that case, the process will inherit Editsync's process environment, and — if the project was opened via ClI — from the CLI.

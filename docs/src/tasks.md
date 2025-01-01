# Tasks

Editsync supports ways to spawn (and rerun) commands using its integrated terminal to output the results. These commands can read a limited subset of Editsync state (such as a path to the file currently being edited or selected text).

```json
[
  {
    "label": "Example task",
    "command": "for i in {1..5}; do echo \"Hello $i/5\"; sleep 1; done",
    //"args": [],
    // Env overrides for the command, will be appended to the terminal's environment from the settings.
    "env": { "foo": "bar" },
    // Current working directory to spawn the command into, defaults to current project root.
    //"cwd": "/path/to/working/directory",
    // Whether to use a new terminal tab or reuse the existing one to spawn the process, defaults to `false`.
    "use_new_terminal": false,
    // Whether to allow multiple instances of the same task to be run, or rather wait for the existing ones to finish, defaults to `false`.
    "allow_concurrent_runs": false,
    // What to do with the terminal pane and tab, after the command was started:
    // * `always` — always show the task's pane, and focus the corresponding tab in it (default)
    // * `no_focus` — always show the task's pane, add the task's tab in it, but don't focus it
    // * `never` — do not alter focus, but still add/reuse the task's tab in its pane
    "reveal": "always",
    // What to do with the terminal pane and tab, after the command had finished:
    // * `never` — Do nothing when the command finishes (default)
    // * `always` — always hide the terminal tab, hide the pane also if it was the last tab in it
    // * `on_success` — hide the terminal tab on task success only, otherwise behaves similar to `always`
    "hide": "never",
    // Which shell to use when running a task inside the terminal.
    // May take 3 values:
    // 1. (default) Use the system's default terminal configuration in /etc/passwd
    //      "shell": "system"
    // 2. A program:
    //      "shell": {
    //        "program": "sh"
    //      }
    // 3. A program with arguments:
    //     "shell": {
    //         "with_arguments": {
    //           "program": "/bin/bash",
    //           "args": ["--login"]
    //         }
    //     }
    "shell": "system",
    // Whether to show the task line in the output of the spawned task, defaults to `true`.
    "show_summary": true,
    // Whether to show the command line in the output of the spawned task, defaults to `true`.
    "show_output": true
  }
]
```

There are two actions that drive the workflow of using tasks: `task: spawn` and `task: rerun`
`task: spawn` opens a modal with all available tasks in the current file.
`task: rerun` reruns the most-recently spawned task. You can also rerun tasks from task modal.

By default, rerunning tasks reuses the same terminal (due to the `"use_new_terminal": false` default) but waits for the previous task to finish before start (due to the `"allow_concurrent_runs": false` default).

Keep `"use_new_terminal": false` and set `"allow_concurrent_runs": true` to allow cancelling previous tasks on rerun.

## Task templates

Tasks can be defined:

- in global `tasks.json` file; such tasks are available in all Editsync projects you work on. This file is usually located in `~/.config/editsync/tasks.json`. You can edit them by using `editsync: open tasks` action.
- in worktree-specific (local) `.editsync/tasks.json` file; such tasks are available only when working on a project with that worktree included. You can edit worktree-specific tasks by using `editsync: open local tasks`.
- on the fly with [oneshot tasks](#oneshot-tasks). These tasks are project-specific and do not persist across sections.
- by language extension.

## Variables

Editsync tasks act just like your shell; that also means that you can reference environmental variables via sh-esque `$VAR_NAME` syntax. A couple of additional environmental variables are set for your convenience.
These variables allow you to pull information from the current editor and use it in your tasks. The following variables are available:

- `EDITSYNC_COLUMN`: current line column
- `EDITSYNC_ROW`: current line row
- `EDITSYNC_FILE`: absolute path of the currently opened file (e.g. `/Users/my-user/path/to/project/src/main.rs`)
- `EDITSYNC_FILENAME`: filename of the currently opened file (e.g. `main.rs`)
- `EDITSYNC_DIRNAME`: absolute path of the currently opened file with file name stripped (e.g. `/Users/my-user/path/to/project/src`)
- `EDITSYNC_RELATIVE_FILE`: path of the currently opened file, relative to `EDITSYNC_WORKTREE_ROOT` (e.g. `src/main.rs`)
- `EDITSYNC_STEM`: stem (filename without extension) of the currently opened file (e.g. `main`)
- `EDITSYNC_SYMBOL`: currently selected symbol; should match the last symbol shown in a symbol breadcrumb (e.g. `mod tests > fn test_task_contexts`)
- `EDITSYNC_SELECTED_TEXT`: currently selected text
- `EDITSYNC_WORKTREE_ROOT`: absolute path to the root of the current worktree. (e.g. `/Users/my-user/path/to/project`)
- `EDITSYNC_CUSTOM_RUST_PACKAGE`: (Rust-specific) name of the parent package of $EDITSYNC_FILE source file.

To use a variable in a task, prefix it with a dollar sign (`$`):

```json
{
  "label": "echo current file's path",
  "command": "echo $EDITSYNC_FILE"
}
```

You can also use verbose syntax that allows specifying a default if a given variable is not available: `${EDITSYNC_FILE:default_value}`

These environmental variables can also be used in tasks `cwd`, `args` and `label` fields.

### Variable Quoting

When working with paths containing spaces or other special characters, please ensure variables are properly escaped.

For example, instead of this (which will fail if the path has a space):

```json
{
  "label": "stat current file",
  "command": "stat $EDITSYNC_FILE"
}
```

Provide the

```json
{
  "label": "stat current file",
  "command": "stat",
  "args": ["$EDITSYNC_FILE"]
}
```

Or explicitly include escaped quotes like so:

```json
{
  "label": "stat current file",
  "command": "stat \"$EDITSYNC_FILE\""
}
```

## Oneshot tasks

The same task modal opened via `task: spawn` supports arbitrary bash-like command execution: type a command inside the modal text field, and use `opt-enter` to spawn it.

Task modal will persist list of those command for current Editsync session, `task: rerun` will also rerun such tasks if they were the last ones spawned.

You can also adjust currently selected task in a modal (`tab` is a default key binding). Doing so will put its command into a prompt that can then be edited & spawned as an oneshot task.

### Ephemeral tasks

You can use cmd modifier when spawning a task via a modal; tasks spawned this way will not have their usage count increased (thus, they will not be respawned with `task: rerun` and they won't be have a high rank in task modal).
The intended use of ephemeral tasks is to stay in the flow with continuous `task: rerun` usage.

## Custom keybindings for tasks

You can define your own keybindings for your tasks via additional argument to `task::Spawn`. If you wanted to bind the aforementioned `echo current file's path` task to `alt-g`, you would add the following snippet in your [`keymap.json`](./key-bindings.md) file:

```json
{
  "context": "Workspace",
  "bindings": {
    "alt-g": ["task::Spawn", { "task_name": "echo current file's path" }]
  }
}
```

Note that these tasks can also have a 'target' specified to control where the spawned task should show up.
This could be useful for launching a terminal application that you want to use in the center area:

```json
// In tasks.json
{
  "label": "start lazygit",
  "command": "lazygit -p $EDITSYNC_WORKTREE_ROOT"
}
```

```json
// In keymap.json
{
  "context": "Workspace",
  "bindings": {
    "alt-g": [
      "task::Spawn",
      { "task_name": "start lazygit", "reveal_target": "center" }
    ]
  }
}
```

## Binding runnable tags to task templates

Editsync supports overriding default action for inline runnable indicators via workspace-local and global `tasks.json` file with the following precedence hierarchy:

1. Workspace `tasks.json`
2. Global `tasks.json`
3. Language-provided tag bindings (default).

To tag a task, add the runnable tag name to `tags` field on task template:

```json
{
  "label": "echo current file's path",
  "command": "echo $EDITSYNC_FILE",
  "tags": ["rust-test"]
}
```

In doing so, you can change which task is shown in runnables indicator.
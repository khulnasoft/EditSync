# OCaml

OCaml support is available through the [OCaml extension](https://github.com/editsync-extensions/ocaml).

- Tree Sitter: [tree-sitter/tree-sitter-ocaml](https://github.com/tree-sitter/tree-sitter-ocaml)
- Language Server: [ocaml/ocaml-lsp](https://github.com/ocaml/ocaml-lsp)

## Setup Instructions

If you have the development environment already setup, you can skip to [Launching Editsync](#launching-editsync)

### Using Opam

Opam is the official package manager for OCaml and is highly recommended for getting started with OCaml. To get started using Opam, please follow the instructions provided [here](https://ocaml.org/install).

Once you install opam and setup a switch with your development environment as per the instructions, you can proceed.

### Launching Editsync

By now you should have `ocamllsp` installed, you can verify so by running

```sh
ocamllsp --help
```

in your terminal. If you get a help message, you're good to go. If not, please revisit the installation instructions for `ocamllsp` and ensure it's properly installed.

With that aside, we can now launch Editsync. Given how the OCaml package manager works, we require you to run Editsync from the terminal, so please make sure you install the [Editsync cli](https://editsync.khulnasoft.com/features#cli) if you haven't already.

Once you have the cli, simply from a terminal, navigate to your project and run

```sh
editsync .
```

Voila! You should have Editsync running with OCaml support, no additional setup required.

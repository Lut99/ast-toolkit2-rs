# ast-toolkit2-rs
A (second) collection of libraries and macros for working with ASTs and compilers in Rust.

This project is a second iteration of the legendary [`ast-toolkit`](https://github.com/Lut99/ast-toolkit-rs)-crate. The reason it is separate is because it is radically different: instead of taking a Rust-centric approach, where the user defines their structs and annotates them with derive macros, this crate takes an AST-centric approach, where you define your AST in a [JSON](https://json.org) file and then generate the applicable Rust structs from it.

If you are curious about why this change was prompted, please refer to [`DESIGN.md`](./DESIGN.md) that attempts to explain the broad points. Or just ask via [issues](https://github.com/Lut99/ast-toolkit2-rs)!


## Repository structure
This repository is divided into two crates:
- [`ast-toolkit2`](./src/lib.rs) is the main crate that anybody _using_ your AST should import and rely on. It defines the interfaces, things like [`Span`]s, etcetera. In principle, you can build a AST-toolkit compliant tree yourself using just this one. However...
- [`ast-toolkit2-build`](./ast-toolkit2-build/src/lib.rs) is a crate that can be imported by [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to automatically generate the tree and associated code based on an AST JSON.


## Usage

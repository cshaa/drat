set positional-arguments

[private]
default:
  @just --list

codegen:
  cd src-tauri; cargo test export_bindings

cargo *args:
  cd src-tauri; cargo "$@"

tauri *args:
  bun tauri "$@"

bun *args:
  bun "$@"

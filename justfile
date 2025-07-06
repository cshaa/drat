set positional-arguments

[private]
default:
  @just --list

dev: codegen (tauri "dev")

codegen:
  cd src-tauri; cargo test export_bindings

cargo *args:
  cd src-tauri; cargo "$@"

tauri *args:
  bun tauri "$@"

bun *args:
  bun "$@"

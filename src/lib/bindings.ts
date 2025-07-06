import { invoke } from "@tauri-apps/api/core";
import type { Command } from "./gen/Command";
import type { CommandResult } from "./gen/CommandResult";
import type { KeysUnion } from "@typek/typek";

export const None = Symbol("None");
export type None = typeof None;

export type CommandName = Command extends infer T
  ? T extends string
    ? T
    : KeysUnion<T>
  : never;

export type CommandArgs<T extends CommandName> = T extends Command
  ? None
  : Extract<Command, { [k in T]: any }>[T];

export type CommandReturnType<T extends CommandName> = Extract<
  CommandResult,
  { [k in T]: any }
>[T];

export async function command<K extends CommandName>(
  which: K,
  args: CommandArgs<K>
): Promise<CommandReturnType<K>> {
  return (
    await invoke<any>(
      "command",
      args === None ? { which } : { which: { [which]: args } }
    )
  )[which];
}

import { Channel, invoke } from "@tauri-apps/api/core";
import type { Command } from "./gen/Command";
import type { CommandResult } from "./gen/CommandResult";
import type { KeysUnion, ObjectEntries } from "@typek/typek";
import type { Event } from "./gen/Event";

export const None = Symbol("None");
export type None = typeof None;

export type VariantsOf<T extends string | {}> = T extends infer S
  ? S extends string
    ? S
    : KeysUnion<S>
  : never;

export type RepresentationOf<
  T extends string | {},
  V extends VariantsOf<T>
> = V extends T ? V : Extract<T, { [k in V]: any }>;

export type InputOf<
  T extends string | {},
  V extends VariantsOf<T>
> = V extends T ? None : Extract<Command, { [_ in V]: any }>[V];

export type OutputOf<T extends string | {}, V extends VariantsOf<T>> = Extract<
  T,
  { [_ in V]: any }
>[V];

export function isA<T extends string | {}, V extends VariantsOf<T>>(
  value: T,
  variant: V
): value is RepresentationOf<T, V> {
  return typeof value === "string"
    ? value === variant
    : variant in (value as object);
}

export function match<T extends string | {}, R>(
  value: T,
  cases: {
    [V in VariantsOf<T>]: (data: OutputOf<T, V>) => R;
  }
): R {
  for (const [v, f] of Object.entries(cases) as [
    VariantsOf<T>,
    (data: any) => R
  ][]) {
    if (isA(value, v)) {
      return f(value[v]);
    }
  }
  throw TypeError(`Case not handled for value: ${JSON.stringify(value)}`);
}

export type CommandName = VariantsOf<Command>;
export type CommandArgs<V extends CommandName> = InputOf<Command, V>;
export type CommandReturnType<V extends CommandName> = OutputOf<Command, V>;

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

export type EventName = VariantsOf<Event>;

export interface EventListenerOptions {
  signal?: AbortSignal;
}

export function addEventListener(
  { signal }: EventListenerOptions,
  fn: (e: Event) => void
) {
  const channel = new Channel<Event>(fn);
  signal?.addEventListener("abort", () => {
    channel.onmessage = () => {};
  });
  invoke<void>("subscribe", { channel });
}

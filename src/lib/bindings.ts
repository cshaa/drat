import { createClient } from "@rspc/client";
import { TauriTransport } from "@rspc/tauri";
import type { Procedures } from "./bindings.gen.ts";

export const client = createClient<Procedures>({
  transport: new TauriTransport(),
});

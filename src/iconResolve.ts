import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core';

/** In-process dedupe so virtualized rows + preview don’t fan out redundant IPC/resvg-era work */
const MEMORY = new Map<string, string | null>();
const IN_FLIGHT = new Map<string, Promise<string | null>>();
const MAX_ENTRIES = 400;

function prune() {
  while (MEMORY.size > MAX_ENTRIES) {
    const first = MEMORY.keys().next();
    if (first.done || first.value === undefined) break;
    MEMORY.delete(first.value);
  }
}

async function invokeOnce(iconName: string): Promise<string | null> {
  try {
    const path = await invoke<string | null>('resolve_desktop_icon_path', {
      name: iconName,
    });
    if (!path) return null;
    try {
      return convertFileSrc(path);
    } catch {
      return null;
    }
  } catch {
    return null;
  }
}

export async function resolveDesktopIconAssetUrl(iconName: string): Promise<string | null> {
  const name = iconName.trim();
  if (!name) return null;
  if (!isTauri()) return null;

  if (MEMORY.has(name)) {
    return MEMORY.get(name)!;
  }

  let job = IN_FLIGHT.get(name);
  if (!job) {
    job = invokeOnce(name);
    IN_FLIGHT.set(name, job);
    void job.finally(() => {
      IN_FLIGHT.delete(name);
    });
  }

  const out = await job;
  MEMORY.set(name, out);
  prune();
  return out;
}

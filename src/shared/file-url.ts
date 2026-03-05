import { readFile } from "@tauri-apps/plugin-fs";

const cache = new Map<string, string>();

/** Reads a local file via Tauri FS plugin and returns a cached blob URL. */
export async function getFileUrl(path: string): Promise<string> {
  if (cache.has(path)) return cache.get(path)!;

  const bytes = await readFile(path);
  const blob = new Blob([bytes]);
  const url = URL.createObjectURL(blob);
  cache.set(path, url);
  return url;
}

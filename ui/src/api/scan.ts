import type { Entry } from "../data/types";

/** Server-derived constants shipped to the UI at boot. */
export interface ScanMeta {
  stale_days: number;
}

export type ScanInfo =
  | {
      status: "scanning";
      elapsed_ms: number;
      items_scanned: number;
      scan_root: string;
    }
  | {
      status: "ready";
      scanned_in_ms: number;
      items_scanned: number;
      scan_root: string;
      tree: Entry;
      meta: ScanMeta;
    }
  | { status: "error"; message: string; scan_root: string };

/** Fetch the current scan state from the duvis server. */
export async function fetchScan(): Promise<ScanInfo> {
  const res = await fetch("/data.json", { cache: "no-store" });
  if (!res.ok) {
    throw new Error(`/data.json returned ${res.status}`);
  }
  return (await res.json()) as ScanInfo;
}

/** Ask the server to start a fresh scan. The next /data.json calls will
 *  report `scanning` again until it completes. */
export async function requestRescan(): Promise<void> {
  const res = await fetch("/rescan", { method: "POST" });
  if (!res.ok && res.status !== 202) {
    throw new Error(`/rescan returned ${res.status}`);
  }
}

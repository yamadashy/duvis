import { humanSize } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";

/** Join the scan root with the segment path using the scan root's own
 *  separator. We sniff `\` vs `/` from `scanRoot` so a Windows scan
 *  reported as `C:\Users\me` gets backslashes, while a Unix scan stays
 *  with forward slashes. */
export function joinPath(scanRoot: string, segments: readonly string[]): string {
  if (segments.length === 0) return scanRoot;
  const sep = scanRoot.includes("\\") && !scanRoot.includes("/") ? "\\" : "/";
  const trimmed = scanRoot.endsWith(sep) ? scanRoot.slice(0, -1) : scanRoot;
  return `${trimmed}${sep}${segments.join(sep)}`;
}

/** Wraps the async clipboard API in a transient ok/error state so the
 *  button label can flash a confirmation. Falls back to the legacy
 *  textarea + execCommand path on browsers without `navigator.clipboard`
 *  (older Safari over plain http, etc.). */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // fall through to the legacy path
  }
  try {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}

/** Build the clipboard payload for a single selected entry. Mirrors the
 *  shape of the CLI's `--json` per-entry record (`relative_path`,
 *  `size_human`, `file_count`, ...) plus the absolute path and the
 *  `pct_of_root` that's always visible in the detail panel — so an agent
 *  receiving a paste has everything needed to act (`du`, `rm`, jq) or
 *  reason about scale (% of root) without a follow-up CLI call.
 *
 *  `children` is intentionally omitted: pasting a selected directory
 *  shouldn't dump its whole subtree. The full tree is already available
 *  via `duvis --json`. */
export function buildEntryPayload(
  node: TreeNode,
  scanRoot: string,
  segments: readonly string[],
  total: number,
): Record<string, unknown> {
  const isDir = !!node.children && node.children.length > 0;
  const size = node.value ?? 0;

  // Match `precompute_subtree_counts` in src/output/mod.rs: file leaves
  // count themselves as 1 file / 0 dirs; directories count themselves as
  // 1 dir, then accumulate descendants. Same semantics across the wire.
  let fileCount = 0;
  let dirCount = 0;
  node.each((n) => {
    const nIsLeaf = !n.children || n.children.length === 0;
    if (nIsLeaf) fileCount += 1;
    else dirCount += 1;
  });

  const payload: Record<string, unknown> = {
    name: node.data.name,
    relative_path: segments.length === 0 ? "." : segments.join("/"),
    absolute_path: joinPath(scanRoot, segments),
    scan_root: scanRoot,
    is_dir: isDir,
    category: node.data.category,
    size,
    size_human: humanSize(size),
    pct_of_root: total > 0 ? Math.round((size / total) * 1000) / 10 : 0,
    depth: node.depth,
  };
  if (node.data.modified_days_ago !== undefined) {
    payload.modified_days_ago = node.data.modified_days_ago;
  }
  if (isDir) {
    payload.child_count = node.children?.length ?? 0;
    payload.file_count = fileCount;
    payload.dir_count = dirCount;
  }
  return payload;
}

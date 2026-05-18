// Mirrors the Rust `Entry` struct serialized by serde_json.
// Keep field names/cases identical to what `src/entry.rs` emits.

export type Category =
  // Core (always shown in the legend, even at 0 bytes).
  | "cache"
  | "build"
  | "log"
  | "media"
  | "vcs"
  | "ide"
  | "other"
  // Extended (shown in the legend only when at least one entry of this
  // category is present in the scan).
  | "archive"
  | "installer"
  | "vm_image"
  | "model_cache"
  | "backup";

export interface Entry {
  name: string;
  size: number;
  is_dir: boolean;
  category: Category;
  /** Days since last modification. Absent when the FS doesn't expose it. */
  modified_days_ago?: number;
  children?: Entry[];
}

export type SortMode = "size" | "oldest" | "newest" | "name";
export type ViewMode = "treemap" | "sunburst" | "list";

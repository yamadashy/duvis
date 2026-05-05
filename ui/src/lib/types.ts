// Mirrors the Rust `Entry` struct serialized by serde_json.
// Keep field names/cases identical to what `src/entry.rs` emits.

export type Category = "cache" | "build" | "log" | "media" | "vcs" | "ide" | "other";

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

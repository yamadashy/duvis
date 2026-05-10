import { useReducer } from "react";
import { ALL_CATEGORIES } from "./lib/categories";
import type { Category, Entry, SortMode, ViewMode } from "./lib/types";

export type Theme = "dark" | "light";

const THEME_STORAGE_KEY = "duvis.theme";

export function loadStoredTheme(): Theme {
  try {
    const v = localStorage.getItem(THEME_STORAGE_KEY);
    if (v === "dark" || v === "light") return v;
  } catch {
    // ignore (private mode etc.)
  }
  return "light";
}

export function persistTheme(theme: Theme): void {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // ignore
  }
}

const COLUMN_WIDTHS_STORAGE_KEY = "duvis.columnWidths";

export interface ColumnWidths {
  left: number;
  right: number;
}

export const COLUMN_DEFAULTS: ColumnWidths = { left: 304, right: 232 };
export const COLUMN_MIN = 180;
export const COLUMN_MAX = 640;

export function loadStoredColumnWidths(): ColumnWidths {
  try {
    const v = localStorage.getItem(COLUMN_WIDTHS_STORAGE_KEY);
    if (v) {
      const parsed = JSON.parse(v) as Partial<ColumnWidths>;
      return {
        left: clampColumn(parsed.left ?? COLUMN_DEFAULTS.left),
        right: clampColumn(parsed.right ?? COLUMN_DEFAULTS.right),
      };
    }
  } catch {
    // ignore
  }
  return { ...COLUMN_DEFAULTS };
}

export function persistColumnWidths(w: ColumnWidths): void {
  try {
    localStorage.setItem(COLUMN_WIDTHS_STORAGE_KEY, JSON.stringify(w));
  } catch {
    // ignore
  }
}

export function clampColumn(n: number): number {
  return Math.max(COLUMN_MIN, Math.min(COLUMN_MAX, Math.round(n)));
}

export const MIN_DEPTH = 1;
// Each view has its own ceiling. Deep treemap is now safe — layoutTreemap
// drops sub-pixel cells and caps render count, so 10 won't hang anymore.
export const MAX_DEPTH_BY_VIEW: Record<ViewMode, number> = {
  treemap: 10,
  sunburst: 10,
  list: 10,
};
const DEPTH_DEFAULTS: Record<ViewMode, number> = {
  treemap: 5,
  sunburst: 5,
  list: 3,
};

export interface AppState {
  data: Entry;
  rootPath: string[];
  selectedPath: string[] | null; // path of names from data root to selected node
  filterCategories: ReadonlySet<Category>;
  /** Free-text search applied alongside the category filter. Trimmed,
   *  lowercased at the comparison site. Empty string = no filter. */
  searchQuery: string;
  sort: SortMode;
  view: ViewMode;
  /** Depth slider value remembered separately per view. */
  depthByView: Record<ViewMode, number>;
  theme: Theme;
}

export type Action =
  | { type: "navigateTo"; path: string[] }
  | { type: "select"; path: string[] | null }
  | {
      type: "toggleCategory";
      category: Category;
      solo?: boolean;
      /** Categories currently rendered in the legend. Hidden extended
       *  categories must not block the "all visible turned off → reset"
       *  rescue. */
      visible: ReadonlySet<Category>;
    }
  | { type: "resetCategories" }
  | { type: "setSearch"; query: string }
  | { type: "setSort"; sort: SortMode }
  | { type: "setView"; view: ViewMode }
  | { type: "setDepth"; depth: number }
  | { type: "toggleTheme" };

export function initialState(data: Entry): AppState {
  return {
    data,
    rootPath: [],
    selectedPath: null,
    filterCategories: new Set(ALL_CATEGORIES),
    searchQuery: "",
    sort: "size",
    view: "treemap",
    depthByView: { ...DEPTH_DEFAULTS },
    theme: loadStoredTheme(),
  };
}

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case "navigateTo":
      return { ...state, rootPath: action.path, selectedPath: null };
    case "select":
      return { ...state, selectedPath: action.path };
    case "toggleCategory": {
      if (action.solo) {
        return { ...state, filterCategories: new Set([action.category]) };
      }
      const next = new Set(state.filterCategories);
      if (next.has(action.category)) next.delete(action.category);
      else next.add(action.category);
      // Avoid a fully-dim UI: if the user toggled off every category they
      // can actually see in the legend, reset to all-on. We check
      // `action.visible` rather than `next.size` because hidden extended
      // categories (no entries in this scan) linger in the set and would
      // otherwise mask the "everything visible is off" condition.
      const anyVisibleActive = Array.from(action.visible).some((c) => next.has(c));
      if (!anyVisibleActive) return { ...state, filterCategories: new Set(ALL_CATEGORIES) };
      return { ...state, filterCategories: next };
    }
    case "resetCategories":
      return { ...state, filterCategories: new Set(ALL_CATEGORIES) };
    case "setSearch":
      return { ...state, searchQuery: action.query };
    case "setSort":
      return { ...state, sort: action.sort };
    case "setView":
      return { ...state, view: action.view };
    case "setDepth": {
      const cap = MAX_DEPTH_BY_VIEW[state.view];
      const clamped = Math.max(MIN_DEPTH, Math.min(cap, action.depth));
      return {
        ...state,
        depthByView: { ...state.depthByView, [state.view]: clamped },
      };
    }
    case "toggleTheme":
      return { ...state, theme: state.theme === "dark" ? "light" : "dark" };
  }
}

export function useAppState(data: Entry) {
  return useReducer(reducer, data, initialState);
}

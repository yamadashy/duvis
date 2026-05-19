const COLUMN_WIDTHS_STORAGE_KEY = "duvis.columnWidths";

export interface ColumnWidths {
  left: number;
  right: number;
}

export const COLUMN_DEFAULTS: ColumnWidths = { left: 304, right: 232 };
export const COLUMN_MIN = 180;
export const COLUMN_MAX = 640;

export function clampColumn(n: number): number {
  return Math.max(COLUMN_MIN, Math.min(COLUMN_MAX, Math.round(n)));
}

export function loadStoredColumnWidths(): ColumnWidths {
  try {
    const v = localStorage.getItem(COLUMN_WIDTHS_STORAGE_KEY);
    if (v) {
      const parsed = JSON.parse(v) as Partial<ColumnWidths>;
      return {
        left: clampColumn(finiteOrDefault(parsed.left, COLUMN_DEFAULTS.left)),
        right: clampColumn(finiteOrDefault(parsed.right, COLUMN_DEFAULTS.right)),
      };
    }
  } catch {
    // ignore
  }
  return { ...COLUMN_DEFAULTS };
}

// Guard against the persisted JSON containing strings, NaN, or other
// non-finite values (older versions, manual edits, schema drift). Letting
// those reach `clampColumn` would produce `NaN`, which then bleeds into
// the CSS variable and collapses the panel column.
function finiteOrDefault(v: unknown, fallback: number): number {
  return typeof v === "number" && Number.isFinite(v) ? v : fallback;
}

export function persistColumnWidths(w: ColumnWidths): void {
  try {
    localStorage.setItem(COLUMN_WIDTHS_STORAGE_KEY, JSON.stringify(w));
  } catch {
    // ignore
  }
}

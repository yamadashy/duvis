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

import type { MouseEvent } from "react";
import { CATEGORIES, categoryVar } from "../lib/categories";
import { humanSize, pct } from "../lib/format";
import type { Category } from "../lib/types";
import "./Legend.css";

interface LegendProps {
  byCategory: Readonly<Record<string, number>>;
  total: number;
  active: ReadonlySet<Category>;
  onToggle: (category: Category, solo: boolean) => void;
}

export function Legend({ byCategory, total, active, onToggle }: LegendProps) {
  return (
    <div className="legend">
      {CATEGORIES.map((c) => {
        const isActive = active.has(c.key);
        const size = byCategory[c.key] ?? 0;
        return (
          <button
            type="button"
            key={c.key}
            className="legend-row"
            data-active={isActive}
            onClick={(e: MouseEvent) => onToggle(c.key, e.shiftKey)}
            title={`${c.label}${tagHint(c.tag)}`}
          >
            <span className="legend-swatch" style={{ background: categoryVar(c.key) }} />
            <span className="legend-label">
              {c.label}
              {c.tag ? <span className="legend-tag">{c.tag}</span> : null}
            </span>
            <span className="legend-size mono tabular">{humanSize(size)}</span>
            <span
              className="legend-pct mono"
              style={{ fontSize: 10, color: "var(--fg-dim)" }}
            >
              {pct(size, total)}
            </span>
            <div className="legend-bar">
              <div
                className="legend-bar-fill"
                style={{ width: pct(size, total), background: categoryVar(c.key) }}
              />
            </div>
          </button>
        );
      })}
    </div>
  );
}

function tagHint(tag: "safe" | "warn" | null): string {
  return tag ? ` — ${tag === "safe" ? "safely deletable" : "rebuildable"}` : "";
}

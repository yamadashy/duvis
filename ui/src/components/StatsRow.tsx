import { humanSize, pct } from "../lib/format";
import type { CategoryAggregates } from "../lib/treemap";
import "./StatsRow.css";

interface StatsRowProps {
  agg: CategoryAggregates;
  itemCount: number;
}

export function StatsRow({ agg, itemCount }: StatsRowProps) {
  return (
    <div className="stats">
      <div className="stat">
        <div className="stat-label">Total scanned</div>
        <div className="stat-value tabular">{humanSize(agg.total)}</div>
        <div className="stat-sub">
          <span className="mono tabular">{itemCount.toLocaleString()}</span> items
        </div>
      </div>

      <div className="stat">
        <div className="stat-label">
          Reclaimable
          <button
            type="button"
            className="stat-help"
            aria-label="What is Reclaimable?"
            title={
              "Total size of categories that are usually safe to delete:\n" +
              "  • cache  (re-downloadable, e.g. node_modules, .cargo, .ollama)\n" +
              "  • build  (re-buildable, e.g. target, dist, .next)\n" +
              "  • log    (old logs typically discardable)\n" +
              "These are estimates — review before deleting."
            }
          >
            ?
          </button>
        </div>
        <div className="stat-value tabular">{humanSize(agg.deletable)}</div>
        <div className="stat-sub">
          <span className="stat-pill mono">{pct(agg.deletable, agg.total)}</span>
          cache + build + log
        </div>
        <div className="stat-bar">
          <span
            style={{
              width: pct(agg.byCategory.cache ?? 0, agg.deletable || 1),
              background: "var(--cat-cache)",
            }}
          />
          <span
            style={{
              width: pct(agg.byCategory.build ?? 0, agg.deletable || 1),
              background: "var(--cat-build)",
            }}
          />
          <span
            style={{
              width: pct(agg.byCategory.log ?? 0, agg.deletable || 1),
              background: "var(--cat-log)",
            }}
          />
        </div>
      </div>

      <div className="stat">
        <div className="stat-label">Stale (&gt; 90d)</div>
        <div className="stat-value tabular">{humanSize(agg.stale)}</div>
        <div className="stat-sub">
          <span className="mono">{pct(agg.stale, agg.total)}</span> untouched
        </div>
      </div>
    </div>
  );
}

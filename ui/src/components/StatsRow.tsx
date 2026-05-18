import { humanSize, pct } from "../data/format";
import type { CategoryAggregates } from "../data/hierarchy";
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
        <div className="stat-label">Files</div>
        <div className="stat-value tabular">{agg.fileCount.toLocaleString()}</div>
        <div className="stat-sub">leaf entries</div>
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

import { humanSize, pct } from "../data/format";
import type { CategoryAggregates } from "../data/hierarchy";
import styles from "./StatsRow.module.css";

interface StatsRowProps {
  agg: CategoryAggregates;
  itemCount: number;
}

export function StatsRow({ agg, itemCount }: StatsRowProps) {
  return (
    <div className={styles.stats}>
      <div className={styles.stat}>
        <div className={styles.statLabel}>Total scanned</div>
        <div className={`${styles.statValue} tabular`}>{humanSize(agg.total)}</div>
        <div className={styles.statSub}>
          <span className="mono tabular">{itemCount.toLocaleString()}</span> items
        </div>
      </div>

      <div className={styles.stat}>
        <div className={styles.statLabel}>Files</div>
        <div className={`${styles.statValue} tabular`}>{agg.fileCount.toLocaleString()}</div>
        <div className={styles.statSub}>leaf entries</div>
      </div>

      <div className={styles.stat}>
        <div className={styles.statLabel}>Stale (&gt; 90d)</div>
        <div className={`${styles.statValue} tabular`}>{humanSize(agg.stale)}</div>
        <div className={styles.statSub}>
          <span className="mono">{pct(agg.stale, agg.total)}</span> untouched
        </div>
      </div>
    </div>
  );
}

import "./ScanningView.css";

interface ScanningViewProps {
  scanRoot: string;
  elapsedMs: number;
  itemsScanned: number;
}

export function ScanningView({ scanRoot, elapsedMs, itemsScanned }: ScanningViewProps) {
  const seconds = Math.floor(elapsedMs / 1000);
  const rate = elapsedMs > 0 ? Math.round((itemsScanned / elapsedMs) * 1000) : 0;
  return (
    <div className="scanning-view">
      <div className="scanning-card">
        <div className="scanning-spinner" aria-hidden="true" />
        <div className="scanning-title">Scanning…</div>
        <div className="scanning-path mono">{scanRoot}</div>
        <div className="scanning-stats mono tabular">
          <span className="scanning-stat">
            <span className="scanning-stat-value">{itemsScanned.toLocaleString()}</span>
            <span className="scanning-stat-label">items</span>
          </span>
          <span className="scanning-stat-sep" aria-hidden="true" />
          <span className="scanning-stat">
            <span className="scanning-stat-value">{seconds}s</span>
            <span className="scanning-stat-label">elapsed</span>
          </span>
          {rate > 0 ? (
            <>
              <span className="scanning-stat-sep" aria-hidden="true" />
              <span className="scanning-stat">
                <span className="scanning-stat-value">{rate.toLocaleString()}</span>
                <span className="scanning-stat-label">items/s</span>
              </span>
            </>
          ) : null}
        </div>
      </div>
    </div>
  );
}

interface ErrorViewProps {
  scanRoot: string;
  message: string;
  onRescan: () => void;
}

export function ErrorView({ scanRoot, message, onRescan }: ErrorViewProps) {
  return (
    <div className="scanning-view">
      <div className="scanning-card">
        <div className="scanning-title scanning-title-error">Scan failed</div>
        <div className="scanning-path mono">{scanRoot}</div>
        <div className="scanning-error">{message}</div>
        <button type="button" className="btn primary" onClick={onRescan}>
          Try again
        </button>
      </div>
    </div>
  );
}

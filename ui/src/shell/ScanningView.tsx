import styles from "./ScanningView.module.css";

interface ScanningViewProps {
  scanRoot: string;
  elapsedMs: number;
  itemsScanned: number;
}

export function ScanningView({ scanRoot, elapsedMs, itemsScanned }: ScanningViewProps) {
  const seconds = Math.floor(elapsedMs / 1000);
  const rate = elapsedMs > 0 ? Math.round((itemsScanned / elapsedMs) * 1000) : 0;
  return (
    <div className={styles.scanningView}>
      <div className={styles.scanningCard}>
        <div className={styles.scanningSpinner} aria-hidden="true" />
        <div className={styles.scanningTitle}>Scanning…</div>
        <div className={`${styles.scanningPath} mono`}>{scanRoot}</div>
        <div className={`${styles.scanningStats} mono tabular`}>
          <span className={styles.scanningStat}>
            <span className={styles.scanningStatValue}>{itemsScanned.toLocaleString()}</span>
            <span className={styles.scanningStatLabel}>items</span>
          </span>
          <span className={styles.scanningStatSep} aria-hidden="true" />
          <span className={styles.scanningStat}>
            <span className={styles.scanningStatValue}>{seconds}s</span>
            <span className={styles.scanningStatLabel}>elapsed</span>
          </span>
          {rate > 0 ? (
            <>
              <span className={styles.scanningStatSep} aria-hidden="true" />
              <span className={styles.scanningStat}>
                <span className={styles.scanningStatValue}>{rate.toLocaleString()}</span>
                <span className={styles.scanningStatLabel}>items/s</span>
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
    <div className={styles.scanningView}>
      <div className={styles.scanningCard}>
        <div className={`${styles.scanningTitle} ${styles.scanningTitleError}`}>Scan failed</div>
        <div className={`${styles.scanningPath} mono`}>{scanRoot}</div>
        <div className={styles.scanningError}>{message}</div>
        <button type="button" className="btn primary" onClick={onRescan}>
          Try again
        </button>
      </div>
    </div>
  );
}

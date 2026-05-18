import { useEffect, useState } from "react";
import { fetchScan, requestRescan, type ScanInfo } from "../api/scan";

const POLL_INTERVAL_MS = 500;

interface UseScanPollingResult {
  scan: ScanInfo;
  /** Trigger a fresh scan and reset the polling effect. */
  rescan: () => void;
}

/** Polls `/data.json` every 500ms while the server reports `scanning`,
 *  stops once it reports `ready` or `error`. Calling `rescan()` posts to
 *  `/rescan` and restarts the polling loop optimistically (the next
 *  fetch will report `scanning` again). */
export function useScanPolling(): UseScanPollingResult {
  const [scan, setScan] = useState<ScanInfo>({
    status: "scanning",
    elapsed_ms: 0,
    items_scanned: 0,
    scan_root: "",
  });
  // Bumping this restarts the polling effect, used right after a manual rescan.
  const [pollEpoch, setPollEpoch] = useState(0);

  useEffect(() => {
    let cancelled = false;
    let timeoutId: number | undefined;

    async function tick() {
      try {
        const info = await fetchScan();
        if (cancelled) return;
        setScan(info);
        if (info.status === "scanning") {
          timeoutId = window.setTimeout(tick, POLL_INTERVAL_MS);
        }
      } catch (err) {
        if (cancelled) return;
        setScan({
          status: "error",
          message: err instanceof Error ? err.message : String(err),
          scan_root: "",
        });
      }
    }

    tick();
    return () => {
      cancelled = true;
      if (timeoutId) clearTimeout(timeoutId);
    };
  }, [pollEpoch]);

  function rescan() {
    requestRescan().catch(() => {
      // The next poll will surface any error.
    });
    setScan((prev) => ({
      status: "scanning",
      elapsed_ms: 0,
      items_scanned: 0,
      scan_root: prev.scan_root,
    }));
    setPollEpoch((n) => n + 1);
  }

  return { scan, rescan };
}

import { relTime } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";
import styles from "./DetailPanel.module.css";

export function MetadataGrid({ node }: { node: TreeNode }) {
  const days = node.data.modified_days_ago;
  return (
    <div className={styles.detailSection}>
      <div className={styles.detailSectionTitle}>Metadata</div>
      <div className={styles.detailMeta}>
        <span className={styles.detailMetaKey}>Type</span>
        <span className={styles.detailMetaVal}>{node.children ? "directory" : "file"}</span>
        <span className={styles.detailMetaKey}>Modified</span>
        <span className={styles.detailMetaVal}>{relTime(days)}</span>
        <span className={styles.detailMetaKey}>Items</span>
        <span className={styles.detailMetaVal}>
          {node.children ? (node.descendants().length - 1).toLocaleString() : "—"}
        </span>
        <span className={styles.detailMetaKey}>Depth from root</span>
        <span className={styles.detailMetaVal}>{node.depth}</span>
      </div>
    </div>
  );
}

import { relTime } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";

export function MetadataGrid({ node }: { node: TreeNode }) {
  const days = node.data.modified_days_ago;
  return (
    <div className="detail-section">
      <div className="detail-section-title">Metadata</div>
      <div className="detail-meta">
        <span className="detail-meta-key">Type</span>
        <span className="detail-meta-val">{node.children ? "directory" : "file"}</span>
        <span className="detail-meta-key">Modified</span>
        <span className="detail-meta-val">{relTime(days)}</span>
        <span className="detail-meta-key">Items</span>
        <span className="detail-meta-val">
          {node.children ? (node.descendants().length - 1).toLocaleString() : "—"}
        </span>
        <span className="detail-meta-key">Depth from root</span>
        <span className="detail-meta-val">{node.depth}</span>
      </div>
    </div>
  );
}

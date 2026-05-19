import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
  info: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  override state: State = { error: null, info: null };

  static getDerivedStateFromError(error: Error): State {
    return { error, info: null };
  }

  override componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("duvis UI error:", error, info);
    this.setState({ error, info });
  }

  override render() {
    if (this.state.error) {
      return (
        <div
          style={{
            padding: 24,
            color: "#fff",
            background: "#0a0b0f",
            fontFamily: "ui-monospace, monospace",
            fontSize: 12,
            whiteSpace: "pre-wrap",
            height: "100vh",
            overflow: "auto",
          }}
        >
          <div style={{ color: "#ef4444", fontSize: 14, marginBottom: 12 }}>duvis UI crashed</div>
          <div style={{ marginBottom: 12 }}>{this.state.error.message}</div>
          <div style={{ color: "#9aa0ad", fontSize: 11 }}>{this.state.error.stack}</div>
          {this.state.info ? (
            <div style={{ color: "#9aa0ad", fontSize: 11, marginTop: 12 }}>
              {this.state.info.componentStack}
            </div>
          ) : null}
        </div>
      );
    }
    return this.props.children;
  }
}

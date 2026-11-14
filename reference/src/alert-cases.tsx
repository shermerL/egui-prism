import * as React from "react";
import { Alert, AlertTitle, AlertDescription } from "./ui/alert";

function Plus() { return <svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M7 2h2v5h5v2H9v5H7V9H2V7h5z" /></svg>; }
export function AlertCases() {
  const [dark, setDark] = React.useState(false);
  const [scale, setScale] = React.useState(1);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  return <main style={{ padding: 32, minHeight: "100vh", background: "var(--background)", color: "var(--foreground)" }}>
    <div className="mb-8 flex gap-4 text-sm">
      <label><input type="checkbox" checked={dark} onChange={e => setDark(e.target.checked)} /> Dark theme</label>
      <button onClick={() => setScale(1)}>100%</button><button onClick={() => setScale(1.5)}>150%</button>
    </div>
    <section style={{ zoom: scale }} className="space-y-4">
      <h1 className="text-xl font-semibold">Alerts</h1>
      <p className="text-sm">Caller-owned inline notices. No activation, focus, timer or dismissal.</p>
      <div style={{ maxWidth: 448 }} className="space-y-2">
        <Alert><Plus/><AlertTitle>Heads up!</AlertTitle><AlertDescription>You can add components to your app using the CLI.</AlertDescription></Alert>
        <Alert variant="destructive"><Plus/><AlertTitle>Unable to save</AlertTitle><AlertDescription>Your changes are still available in this window.</AlertDescription></Alert>
        <Alert><AlertTitle>A title without a description or icon</AlertTitle></Alert>
        <Alert><Plus/><AlertDescription>A description without a title.</AlertDescription></Alert>
      </div>
      <p className="text-sm">160px content: title elides, description wraps, full text remains accessible</p>
      <Alert style={{ width: 160 }}><AlertTitle>A deliberately long title for a bounded notice</AlertTitle><AlertDescription>A long description wraps onto several lines inside the available width and remains accessible.</AlertDescription></Alert>
      <p className="text-sm">320px notice clipped to 54px height</p>
      <div style={{ width: 320, height: 54, overflow: "hidden" }}><Alert><AlertTitle>Visible title</AlertTitle><AlertDescription>This longer description continues beyond the visible parent clip. It remains complete in accessibility metadata.</AlertDescription></Alert></div>
      <p className="text-sm">Parent opacity 60% over a tinted surface</p>
      <div style={{ background: "#3478f6", padding: 12, width: 344 }}><div style={{ opacity: 0.6 }}><Alert variant="destructive"><AlertTitle>Shared opacity</AlertTitle><AlertDescription>Text, border and background composite together.</AlertDescription></Alert></div></div>
    </section>
  </main>;
}

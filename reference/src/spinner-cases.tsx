import * as React from "react";
import { Spinner } from "./ui/spinner";
import { Button } from "./ui/button";

export function SpinnerCases() {
  const [dark, setDark] = React.useState(false);
  const [hidden, setHidden] = React.useState(false);
  const [reduced, setReduced] = React.useState(false);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  // Explicit caller policy, not a change to the pinned Spinner source.
  const style = reduced ? { animation: "none" } : undefined;
  return <main style={{ padding: 32, background: "var(--background)", color: "var(--foreground)", minHeight: "100vh" }}>
    <h1>Spinners</h1>
    <div className="flex gap-4 my-4">
      <label><input type="checkbox" checked={dark} onChange={e => setDark(e.target.checked)} /> Dark theme</label>
      <label><input type="checkbox" checked={hidden} onChange={e => setHidden(e.target.checked)} /> Hide indicators</label>
      <label><input type="checkbox" checked={reduced} onChange={e => setReduced(e.target.checked)} /> Reduced motion (caller policy)</label>
    </div>
    {!hidden && <div className="space-y-4">
      <div className="flex items-center gap-2"><Spinner aria-label="Loading projects" style={style} /><span>Loading projects</span></div>
      <div className="flex items-center gap-2"><Button disabled>Saving</Button><Spinner aria-label="Saving" style={style} /></div>
      <div style={{background:"#3478f6",padding:12}}><div style={{opacity:0.6}}><Spinner aria-label="Loading preview" style={style} /></div></div>
    </div>}
    <p>Use actual browser zoom for 100%/150%. Caller controls visibility and motion; no progress value is implied.</p>
  </main>;
}

import * as React from "react";
import { Separator } from "./ui/separator";

export function SeparatorCases() {
  const [dark, setDark] = React.useState(false);
  const [scale, setScale] = React.useState(1);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  return <main style={{ padding: 32, minHeight: "100vh", background: "var(--background)", color: "var(--foreground)" }}>
    <div className="mb-8 flex gap-4 text-sm">
      <label><input type="checkbox" checked={dark} onChange={e => setDark(e.target.checked)} /> Dark theme</label>
      <button onClick={() => setScale(1)}>100%</button>
      <button onClick={() => setScale(1.5)}>150%</button>
    </div>
    <section style={{ zoom: scale, width: 320 }} className="space-y-2 text-sm">
      <h1 className="text-xl font-semibold">Separators</h1>
      <p>One-point borders; no activation or keyboard focus.</p>
      {[true, false].map(decorative => <React.Fragment key={String(decorative)}>
        <p>{decorative ? "Decorative" : "Semantic"}</p>
        <Separator decorative={decorative} />
        <div style={{ height: 48 }} className="flex items-center gap-2">
          <span>Left</span><Separator orientation="vertical" decorative={decorative} /><span>Right</span>
        </div>
      </React.Fragment>)}
      <p>320pt separator clipped to 160pt</p>
      <div style={{ width: 160, overflow: "hidden" }}><div style={{ width: 320 }}><Separator /></div></div>
      <p>Parent opacity 60% over a tinted surface</p>
      <div style={{ padding: 12, background: "#3478f6" }}><div style={{ opacity: 0.6 }}><Separator /></div></div>
    </section>
  </main>;
}

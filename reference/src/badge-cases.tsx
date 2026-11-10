import * as React from "react";
import { Badge } from "./ui/badge";

const variants = ["default", "secondary", "destructive", "outline", "ghost", "link"] as const;

export function BadgeCases() {
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
    <section style={{ zoom: scale }} className="space-y-4">
      <h1 className="text-xl font-semibold">Badges</h1>
      <p className="text-sm">Static text; no activation or focus behavior.</p>
      <div className="flex flex-wrap items-center gap-2">
        {variants.map(variant => <Badge key={variant} variant={variant}>{variant === "default" ? "Primary" : variant[0].toUpperCase() + variant.slice(1)}</Badge>)}
      </div>
      <p className="text-sm">Long text inside a 320px horizontal scroll area</p>
      <div style={{ width: 320, overflowX: "auto" }}><Badge variant="outline">A deliberately long badge label stays on one line and remains available by scrolling</Badge></div>
      <p className="text-sm">Parent opacity 60% over a tinted surface</p>
      <div style={{ background: "#3478f6", padding: 12, width: "fit-content" }}>
        <div style={{ opacity: 0.6 }} className="flex gap-2"><Badge>Primary</Badge><Badge variant="destructive">Destructive</Badge><Badge variant="outline">Outline</Badge></div>
      </div>
    </section>
  </main>;
}

import * as React from "react";
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "./ui/card";
import { Button } from "./ui/button";

export function CardCases() {
  const [dark, setDark] = React.useState(false);
  const [scale, setScale] = React.useState(1);
  const [narrow, setNarrow] = React.useState(false);
  const [long, setLong] = React.useState(false);
  const [disabled, setDisabled] = React.useState(false);
  const [text, setText] = React.useState("");
  const [count, setCount] = React.useState(0);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  return <main style={{ padding: 32, minHeight: "100vh", background: "var(--background)", color: "var(--foreground)" }}>
    <div className="mb-8 flex gap-4 text-sm">
      <label><input type="checkbox" checked={dark} onChange={e => setDark(e.target.checked)} /> Dark theme</label>
      <label><input type="checkbox" checked={narrow} onChange={e => setNarrow(e.target.checked)} /> 320px content</label>
      <button onClick={() => setScale(1)}>100%</button><button onClick={() => setScale(1.5)}>150%</button>
    </div>
    <section style={{ zoom: scale, width: narrow ? 320 : 640 }} className="space-y-4">
      <h1 className="text-xl font-semibold">Cards</h1>
      <p className="text-sm">Caller-owned sections, text and actions. Native editor appearance is separate.</p>
      <div className="flex gap-4 text-sm"><label><input type="checkbox" checked={disabled} onChange={e => setDisabled(e.target.checked)} /> Disable nested action</label><label><input type="checkbox" checked={long} onChange={e => setLong(e.target.checked)} /> Long header and body</label></div>
      <Card>
        <CardHeader><CardTitle>{long ? "A deliberately long card title wraps within the caller's narrow layout" : "Project"}</CardTitle><CardDescription>Caller-owned content and actions.</CardDescription></CardHeader>
        <CardContent>{long && Array.from({length: 8}, (_, i) => <p key={i} className="text-sm mb-2">Caller-owned content row {i + 1}</p>)}<input className="border" aria-label="Project text" value={text} onChange={e => setText(e.target.value)} /></CardContent>
        <CardFooter><Button disabled={disabled} onClick={() => setCount(count + 1)}>Save</Button><span className="ml-3 text-sm">Saved: {count}</span></CardFooter>
      </Card>
      <p className="text-sm">Empty Card</p><Card />
      <p className="text-sm">60% isolated shell and header over a tinted surface</p>
      <div style={{ background: "#3478f6", padding: 12 }}><Card style={{ opacity: 0.6 }}><CardHeader><CardTitle>Preview</CardTitle><CardDescription>Two-layer shadow-sm.</CardDescription></CardHeader></Card></div>
    </section>
  </main>;
}

import * as React from "react";
import { Button } from "./ui/button";
import { Empty, EmptyHeader, EmptyTitle, EmptyDescription, EmptyMedia, EmptyContent } from "./ui/empty";

export function EmptyCases() {
  const [dark, setDark] = React.useState(false);
  const [state, setState] = React.useState("empty");
  const [long, setLong] = React.useState(false);
  const [disabled, setDisabled] = React.useState(false);
  const [narrow, setNarrow] = React.useState(false);
  const [count, setCount] = React.useState(0);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  const action = () => { setCount(count + 1); setState("loading"); };
  return <main style={{ padding: 32, minHeight: "100vh", background: "var(--background)", color: "var(--foreground)" }}>
    <div className="mb-8 flex flex-wrap gap-4 text-sm">
      <label><input type="checkbox" checked={dark} onChange={e => setDark(e.target.checked)} /> Dark theme</label>
      <label><input type="checkbox" checked={narrow} onChange={e => setNarrow(e.target.checked)} /> 320px content</label>
      <span>Use browser zoom for 100%/150%, and viewport widths 767px/768px to test md:p-12.</span>
    </div>
    <section style={{ width: narrow ? 320 : "100%", maxWidth: "100%" }} className="space-y-4">
      <h1 className="text-xl font-semibold">Empty states</h1>
      <p className="text-sm">Caller selects empty, loading or failed state. Actions only change local demo state.</p>
      <div className="flex flex-wrap gap-4 text-sm">
        {[["empty","Empty"],["loading","Loading"],["error","Inline error"]].map(([value,label]) => <button key={value} onClick={() => setState(value)} aria-pressed={state===value}>{label}</button>)}
        <label><input type="checkbox" checked={long} onChange={e => setLong(e.target.checked)} /> Long localized copy</label>
        <label><input type="checkbox" checked={disabled} onChange={e => setDisabled(e.target.checked)} /> Disable caller action</label>
      </div>
      <p className="text-sm">Padding follows viewport width: 24px below 768px, 48px at or above it, including in a narrow container.</p>
      {state === "empty" ? <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M11 4h2v7h7v2h-7v7h-2v-7H4v-2h7z"/></svg></EmptyMedia>
          <EmptyTitle>{long ? "Noch keine gespeicherten Projekte vorhanden" : "No projects yet"}</EmptyTitle>
          <EmptyDescription>{long ? "Vos projets enregistrés apparaîtront ici. Créez votre premier projet ou importez un projet existant pour commencer à travailler ensemble." : "Create a project to get started."}</EmptyDescription>
        </EmptyHeader>
        <EmptyContent><Button disabled={disabled} onClick={action}>Create project</Button></EmptyContent>
      </Empty> : state === "loading" ? <p>Loading projects... The caller owns completion or cancellation.</p> : <div className="space-y-4"><p className="text-destructive">Projects could not be loaded. Your existing work is unchanged.</p><Button disabled={disabled} onClick={action}>Retry</Button></div>}
      <p className="text-sm">Caller actions: {count}</p>
      <p className="text-sm">140px container, short title and unbreakable text under the parent clip</p>
      <div style={{width:140}}><Empty><EmptyHeader><EmptyTitle>No items</EmptyTitle><EmptyDescription>UnbreakableProjectIdentifier</EmptyDescription></EmptyHeader></Empty></div>
      <p className="text-sm">60% opacity over a tinted surface</p>
      <div style={{background:"#3478f6",padding:12}}><Empty style={{opacity:0.6}}><EmptyHeader><EmptyTitle>No items</EmptyTitle><EmptyDescription>Nothing has been added yet.</EmptyDescription></EmptyHeader></Empty></div>
    </section>
  </main>;
}

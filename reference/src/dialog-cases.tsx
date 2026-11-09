import * as React from "react";
import { Button } from "./ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "./ui/dialog";

export function DialogCases() {
  const [dark, setDark] = React.useState(false);
  const [long, setLong] = React.useState(false);
  const [reverse, setReverse] = React.useState(false);
  const [keepOpen, setKeepOpen] = React.useState(false);
  const [tinted, setTinted] = React.useState(false);
  const [open, setOpen] = React.useState([false, false]);
  const [actions, setActions] = React.useState([0, 0]);
  const [dismissals, setDismissals] = React.useState([0, 0]);
  const [outside, setOutside] = React.useState(0);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  const setInstanceOpen = (index:number, value:boolean) => {
    if (!value) {
      setDismissals(old => old.map((count, i) => i === index ? count + 1 : count));
      if (keepOpen) return;
    }
    setOpen(old => old.map((state, i) => i === index ? value : state));
  };
  return <main className="space-y-6" style={{padding:32, minHeight:"100vh", background:tinted ? "#3478f6" : "var(--background)", color:"var(--foreground)"}}>
    <h1 className="text-xl font-semibold">shadcn Dialog comparison</h1>
    <p className="text-sm">Pinned shadcn 8a35366 · controlled open state</p>
    <Button variant="outline" onClick={() => setDark(!dark)}>{dark ? "Light" : "Dark"}</Button>
    <div className="flex flex-wrap gap-4 text-sm">
      {[["Long body", long, setLong], ["Reverse instances", reverse, setReverse], ["Decline dismissal", keepOpen, setKeepOpen], ["Tinted surface", tinted, setTinted]].map(([label, value, set]) => <label key={String(label)}>
        <input type="checkbox" checked={Boolean(value)} onChange={event => (set as React.Dispatch<React.SetStateAction<boolean>>)(event.target.checked)} /> {String(label)}
      </label>)}
    </div>
    <div className="flex flex-wrap gap-4">
      {(reverse ? [1, 0] : [0, 1]).map(index => <section className="space-y-2" key={index}>
        <Dialog open={open[index]} onOpenChange={value => setInstanceOpen(index, value)}>
          <DialogTrigger asChild><Button variant="outline">Open dialog {index === 0 ? "A" : "B"}</Button></DialogTrigger>
          <DialogContent style={{maxHeight:"calc(100dvh - 32px)", gridTemplateRows:"auto minmax(0, 1fr)"}}>
            <DialogHeader><DialogTitle>Dialog {index === 0 ? "A" : "B"}</DialogTitle><DialogDescription>Actions belong to the application.</DialogDescription></DialogHeader>
            <div className="min-h-0 overflow-y-auto space-y-4">
              {long && <div className="space-y-2 text-sm">{Array.from({length:30}, (_, row) => <p key={row}>Row {String(row + 1).padStart(2,"0")}</p>)}</div>}
              <div className="flex flex-col items-start gap-2 sm:flex-row-reverse">
                <Button onClick={() => setActions(old => old.map((count, i) => i === index ? count + 1 : count))}>Apply</Button>
                <Button variant="outline" onClick={() => setInstanceOpen(index, false)}>Done</Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
        <output className="block text-sm">Actions: {actions[index]} · Dismissals: {dismissals[index]}</output>
      </section>)}
    </div>
    <Button variant="outline" onClick={() => setOutside(count => count + 1)}>Outside action: {outside}</Button>
  </main>;
}

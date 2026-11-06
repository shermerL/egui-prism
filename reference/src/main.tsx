import * as React from "react";
import { createRoot } from "react-dom/client";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Checkbox } from "./ui/checkbox";
import { Switch } from "./ui/switch";
import { Slider } from "./ui/slider";
import "./reference.css";
function App() {
 const [dark,setDark]=React.useState(false); const [amount,setAmount]=React.useState([50]);
 return <main className={dark?"dark":""} style={{background:"var(--background)",color:"var(--foreground)",minHeight:"100vh",padding:32}}>
 <div className="mx-auto max-w-3xl space-y-8">
 <header className="flex items-center justify-between"><div><h1 className="text-xl font-semibold">shadcn reference</h1><p className="text-sm text-muted-foreground">Pinned source · new-york-v4 · 8a35366</p></div><Button variant="outline" onClick={()=>setDark(!dark)}>{dark?"Light":"Dark"}</Button></header>
 <section className="space-y-4"><h2 className="text-sm font-medium">Buttons</h2><div className="flex flex-wrap gap-3"><Button>Primary</Button><Button variant="secondary">Secondary</Button><Button variant="outline">Outline</Button><Button variant="ghost">Ghost</Button><Button variant="destructive">Destructive</Button><Button disabled>Disabled</Button></div></section>
 <section className="grid grid-cols-2 gap-6"><div className="space-y-2"><label htmlFor="input" className="text-sm font-medium">Input</label><Input id="input" placeholder="Enter a value" /></div><div className="space-y-2"><label htmlFor="invalid" className="text-sm font-medium">Invalid</label><Input id="invalid" aria-invalid="true" defaultValue="Invalid value" /></div></section>
 <section className="flex flex-wrap items-center gap-8"><label className="flex items-center gap-2 text-sm"><Checkbox /> Checkbox</label><label className="flex items-center gap-2 text-sm"><Checkbox defaultChecked /> Checked</label><label className="flex items-center gap-2 text-sm"><Switch /> Switch</label><label className="flex items-center gap-2 text-sm"><Switch defaultChecked /> On</label></section>
 <section className="max-w-md space-y-5"><div className="flex justify-between text-sm font-medium"><label htmlFor="amount">Amount</label><output>{amount[0]}</output></div><Slider id="amount" aria-label="Amount" min={0} max={100} step={5} value={amount} onValueChange={setAmount} /><Slider aria-label="Disabled amount" defaultValue={[35]} max={100} disabled /></section>
 </div></main>;
}
createRoot(document.getElementById("root")!).render(<App/>);

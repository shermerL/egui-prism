import * as React from "react";
import { Compositing } from "./compositing";
import { createRoot } from "react-dom/client";
import { Button } from "./ui/button";
import { ButtonCases } from "./button-cases";
import { InputCases } from "./input-cases";
import { Input } from "./ui/input";
import { Checkbox } from "./ui/checkbox";
import { Switch } from "./ui/switch";
import { Slider } from "./ui/slider";
import { Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "./ui/dialog";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "./ui/select";
import "./reference.css";
import { SelectCases } from "./select-cases";
import { SliderCases } from "./slider-cases";
import { SwitchCases } from "./switch-cases";
function App() {
 const [dark,setDark]=React.useState(false); const [amount,setAmount]=React.useState([50]);
 React.useLayoutEffect(() => {
   // Radix popups portal to body, so the root must own the theme class.
   document.documentElement.classList.toggle("dark", dark);
   return () => document.documentElement.classList.remove("dark");
 }, [dark]);
 return <main style={{background:"var(--background)",color:"var(--foreground)",minHeight:"100vh",padding:32}}>
 <div className="mx-auto max-w-3xl space-y-8">
 <header className="flex items-center justify-between"><div><h1 className="text-xl font-semibold">shadcn reference</h1><p className="text-sm text-muted-foreground">Pinned source · new-york-v4 · 8a35366</p></div><Button variant="outline" onClick={()=>setDark(!dark)}>{dark?"Light":"Dark"}</Button></header>
 <ButtonCases />
 <section className="grid grid-cols-2 gap-6"><div className="space-y-2"><label htmlFor="input" className="text-sm font-medium">Input</label><Input id="input" placeholder="Enter a value" /></div><div className="space-y-2"><label htmlFor="invalid" className="text-sm font-medium">Invalid</label><Input id="invalid" aria-invalid="true" defaultValue="Invalid value" /></div></section>
 <section className="flex flex-wrap items-center gap-8"><label className="flex items-center gap-2 text-sm"><Checkbox /> Checkbox</label><label className="flex items-center gap-2 text-sm"><Checkbox defaultChecked /> Checked</label><label className="flex items-center gap-2 text-sm"><Switch /> Switch</label><label className="flex items-center gap-2 text-sm"><Switch defaultChecked /> On</label></section>
 <SwitchCases />
 <section className="max-w-md space-y-5"><div className="flex justify-between text-sm font-medium"><label htmlFor="amount">Amount</label><output>{amount[0]}</output></div><Slider id="amount" aria-label="Amount" min={0} max={100} step={5} value={amount} onValueChange={setAmount} /><Slider aria-label="Disabled amount" defaultValue={[35]} max={100} disabled /></section>
 <section className="flex gap-4"><Select><SelectTrigger className="w-48"><SelectValue placeholder="Choose an option" /></SelectTrigger><SelectContent><SelectItem value="one">Option one</SelectItem><SelectItem value="two">Option two</SelectItem></SelectContent></Select><Dialog><DialogTrigger asChild><Button variant="outline">Open dialog</Button></DialogTrigger><DialogContent><DialogHeader><DialogTitle>Reference dialog</DialogTitle><DialogDescription>Compare popup geometry, focus and animation.</DialogDescription></DialogHeader></DialogContent></Dialog></section>
 </div></main>;
}
const query = new URLSearchParams(location.search);
createRoot(document.getElementById("root")!).render(query.has("inputs") ? <InputCases/> : query.has("selects") ? <SelectCases/> : query.has("sliders") ? <SliderCases/> : query.has("compositing") ? <Compositing/> : <App/>);

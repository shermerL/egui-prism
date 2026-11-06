import * as React from "react";
import { Slider } from "./ui/slider";
import { Button } from "./ui/button";

export function SliderCases() {
  const [dark, setDark] = React.useState(false);
  const [width, setWidth] = React.useState(448);
  const [zoom, setZoom] = React.useState(1);
  const [tinted, setTinted] = React.useState(false);
  const [disabled, setDisabled] = React.useState(false);
  const [values, setValues] = React.useState([[50], [0]]);
  const [changes, setChanges] = React.useState([0, 0]);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  return <main style={{ padding: 32, zoom }} className="space-y-6">
    <h1 className="text-xl font-semibold">shadcn Slider comparison</h1>
    <p className="text-sm">Pinned shadcn 8a35366 · Radix Slider 1.3.6 · horizontal LTR</p>
    <div className="flex flex-wrap gap-3">
      <Button variant="outline" onClick={() => setDark(!dark)}>{dark ? "Light" : "Dark"}</Button>
      <Button variant="outline" onClick={() => setDisabled(!disabled)}>{disabled ? "Enable" : "Disable"}</Button>
      <Button variant="outline" onClick={() => setTinted(!tinted)}>Tinted surface</Button>
      {[448, 200, 20, 16].map(w => <Button key={w} variant="outline" onClick={() => setWidth(w)}>{w}px</Button>)}
      {[1, 1.5].map(z => <Button key={z} variant="outline" onClick={() => setZoom(z)}>{z * 100}%</Button>)}
    </div>
    <p className="text-sm">Arrows: one step; Shift+Arrows / PageUp / PageDown: ten; Home / End: bounds.</p>
    <div className="space-y-6" style={{ padding: 16, background: tinted ? "#193341" : "var(--background)" }}>
    {[0, 1].map(index => <section key={index} className="space-y-4" style={{ width, minWidth: 16 }}>
      <label className="block text-sm font-medium">Value {index === 0 ? "A" : "B"}</label>
      <Slider aria-label={`Value ${index === 0 ? "A" : "B"}`} value={values[index]} min={index === 0 ? 0 : -1} max={index === 0 ? 100 : 1} step={index === 0 ? 5 : 0.1} disabled={disabled}
        onValueChange={next => {setValues(current => current.map((v, i) => i === index ? next : v)); setChanges(current => current.map((v, i) => i === index ? v + 1 : v));}} />
      <output className="block whitespace-nowrap text-sm">{index === 0 ? "A" : "B"}: {values[index][0].toFixed(1)} · Changes: {changes[index]}</output>
    </section>)}
    <section style={{ width: 448 }} className="space-y-4"><label className="block text-sm font-medium">Disabled amount</label><Slider aria-label="Disabled amount" defaultValue={[35]} disabled /></section>
    </div>
  </main>;
}

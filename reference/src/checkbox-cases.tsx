import * as React from "react";
import { Checkbox } from "./ui/checkbox";

export function CheckboxCases() {
  const [values, setValues] = React.useState([false, true]);
  const [changes, setChanges] = React.useState([0, 0]);
  const [disabled, setDisabled] = React.useState(false);
  const [invalid, setInvalid] = React.useState(false);
  const [reversed, setReversed] = React.useState(false);
  const [compositing, setCompositing] = React.useState(false);
  const [long, setLong] = React.useState(false);
  return <section className="space-y-4" aria-label="Checkbox comparison">
    <h2 className="text-sm font-medium">Checkbox comparison</h2>
    <div className="flex flex-wrap gap-4 text-sm">
      {[["Disable checkboxes", disabled, setDisabled], ["Invalid checkboxes", invalid, setInvalid], ["Reverse checkbox instances", reversed, setReversed], ["Long labels", long, setLong], ["Compositing samples", compositing, setCompositing]].map(([name, value, set]) =>
        <label key={String(name)}><input type="checkbox" checked={Boolean(value)} onChange={e => (set as React.Dispatch<React.SetStateAction<boolean>>)(e.target.checked)} /> {String(name)}</label>)}
    </div>
    {(reversed ? [1, 0] : [0, 1]).map(index => <div key={index} className="space-y-2">
      <label className="flex w-fit max-w-full items-center gap-2 text-sm leading-5">
        <Checkbox disabled={disabled} aria-invalid={invalid || undefined} checked={values[index]} onCheckedChange={value => {
          setValues(old => old.map((v, i) => i === index ? value === true : v));
          setChanges(old => old.map((v, i) => i === index ? v + 1 : v));
        }} />
        <span>{long ? `Enable option ${index === 0 ? "A" : "B"} with a deliberately long label that wraps inside narrow content and remains part of the same clickable control` : `Enable option ${index === 0 ? "A" : "B"}`}</span>
      </label>
      <p className="text-sm">{index === 0 ? "A" : "B"}: {String(values[index])} · Changes: {changes[index]}</p>
    </div>)}
    {compositing && ["#ffffff", "#808080", "#3478f6"].map(background => <div key={background} style={{background, padding:12, display:"flex", gap:8, width:"fit-content"}}>
      {[false,true].map(value => <label key={String(value)} className="flex items-center gap-2 text-sm leading-5"><Checkbox disabled checked={value} /><span>{value ? "On" : "Off"}</span></label>)}
    </div>)}
  </section>;
}

export function CheckboxFixture() {
  const [dark, setDark] = React.useState(false);
  const [narrow, setNarrow] = React.useState(false);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  return <main style={{ background: "var(--background)", color: "var(--foreground)", minHeight: "100vh", padding: 32 }}>
    <div className="mb-8 flex gap-4 text-sm">
      <label><input type="checkbox" checked={dark} onChange={e => setDark(e.target.checked)} /> Dark theme</label>
      <label><input type="checkbox" checked={narrow} onChange={e => setNarrow(e.target.checked)} /> 320px content</label>
    </div>
    <div style={{ width: narrow ? 320 : 768, maxWidth: "100%" }}><CheckboxCases /></div>
  </main>;
}

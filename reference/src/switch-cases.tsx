import * as React from "react";
import { Switch } from "./ui/switch";

export function SwitchCases() {
  const [values, setValues] = React.useState([false, true]);
  const [changes, setChanges] = React.useState([0, 0]);
  const [disabled, setDisabled] = React.useState(false);
  const [small, setSmall] = React.useState(false);
  const [invalid, setInvalid] = React.useState(false);
  const [reversed, setReversed] = React.useState(false);
  const [compositing, setCompositing] = React.useState(false);
  const [long, setLong] = React.useState(false);
  return <section className="space-y-4" aria-label="Switch comparison">
    <h2 className="text-sm font-medium">Switch comparison</h2>
    <div className="flex flex-wrap gap-4 text-sm">
      {[["Disable switches", disabled, setDisabled], ["Small switches", small, setSmall], ["Invalid switches", invalid, setInvalid], ["Reverse switch instances", reversed, setReversed], ["Long labels", long, setLong], ["Compositing samples", compositing, setCompositing]].map(([name, value, set]) =>
        <label key={String(name)}><input type="checkbox" checked={Boolean(value)} onChange={e => (set as React.Dispatch<React.SetStateAction<boolean>>)(e.target.checked)} /> {String(name)}</label>)}
    </div>
    {(reversed ? [1, 0] : [0, 1]).map(index => <div key={index} className="space-y-2">
      <label className="flex w-fit max-w-full items-center gap-2 text-sm leading-5">
        <Switch size={small ? "sm" : "default"} disabled={disabled} aria-invalid={invalid || undefined} checked={values[index]} onCheckedChange={value => {
          setValues(old => old.map((v, i) => i === index ? value : v));
          setChanges(old => old.map((v, i) => i === index ? v + 1 : v));
        }} />
        <span>{long ? `Enable option ${index === 0 ? "A" : "B"} with a deliberately long label that wraps inside narrow content and remains part of the same clickable control` : `Enable option ${index === 0 ? "A" : "B"}`}</span>
      </label>
      <p className="text-sm">{index === 0 ? "A" : "B"}: {String(values[index])} · Changes: {changes[index]}</p>
    </div>)}
    {compositing && ["#ffffff", "#808080", "#3478f6"].map(background => <div key={background} style={{background, padding:12, display:"flex", gap:8, width:"fit-content"}}>
      {[false,true].map(value => <label key={String(value)} className="flex items-center gap-2 text-sm leading-5"><Switch size={small ? "sm" : "default"} disabled checked={value} /><span>{value ? "On" : "Off"}</span></label>)}
    </div>)}
  </section>;
}

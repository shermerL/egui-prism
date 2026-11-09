import * as React from "react";
import { Button } from "./ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";

const shortItems = [
  {value:"apple", label:"Apple", disabled:false},
  {value:"apricot", label:"Apricot", disabled:false},
  {value:"banana", label:"Banana", disabled:true},
  {value:"blueberry", label:"Blueberry", disabled:false},
  {value:"cherry", label:"Cherry", disabled:false},
];
const longItems = Array.from({length:30}, (_, index) => ({
  value:`option-${index + 1}`, label:`Option ${String(index + 1).padStart(2, "0")}`, disabled:index === 6 || index === 13,
}));

export function SelectCases() {
  const [dark, setDark] = React.useState(false);
  const [small, setSmall] = React.useState(false);
  const [disabled, setDisabled] = React.useState(false);
  const [invalid, setInvalid] = React.useState(false);
  const [long, setLong] = React.useState(false);
  const [reverse, setReverse] = React.useState(false);
  const [empty, setEmpty] = React.useState(false);
  const [allDisabled, setAllDisabled] = React.useState(false);
  const [tinted, setTinted] = React.useState(false);
  const [values, setValues] = React.useState(["", "cherry"]);
  const [changes, setChanges] = React.useState([0, 0]);
  const [outside, setOutside] = React.useState(0);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => {
      document.documentElement.classList.remove("dark");
    };
  }, [dark]);
  const items = empty ? [] : long ? longItems : shortItems;
  const select = (index:number, alwaysDisabled=false) => <Select value={values[index]} disabled={disabled || alwaysDisabled} onValueChange={value => {
    setValues(old => old.map((v, i) => i === index ? value : v));
    setChanges(old => old.map((v, i) => i === index ? v + 1 : v));
  }}>
    <SelectTrigger aria-label={`Value ${index === 0 ? "A" : "B"}${alwaysDisabled ? " disabled" : ""}`} size={small ? "sm" : "default"} aria-invalid={invalid || undefined} className="w-48">
      <SelectValue placeholder="Choose an option" />
    </SelectTrigger>
    <SelectContent>{items.map(item => <SelectItem key={item.value} value={item.value} disabled={item.disabled || allDisabled}>{item.label}</SelectItem>)}</SelectContent>
  </Select>;
  return <main style={{padding:32, minHeight:"100vh", background:"var(--background)", color:"var(--foreground)"}} className="space-y-6">
    <h1 className="text-xl font-semibold">shadcn Select comparison</h1>
    <p className="text-sm">Pinned shadcn 8a35366 · Radix Select 2.2.6 · item-aligned</p>
    <div className="flex flex-wrap gap-3">
      <Button variant="outline" onClick={() => setDark(!dark)}>{dark ? "Light" : "Dark"}</Button>
      <Button variant="outline" onClick={() => setValues(["", long ? "option-30" : "cherry"])}>Reset values</Button>
    </div>
    <div className="flex flex-wrap gap-4 text-sm">
      {[["Small", small, setSmall], ["Disabled", disabled, setDisabled], ["Invalid", invalid, setInvalid], ["Long list", long, setLong], ["Reverse instances", reverse, setReverse], ["Empty list", empty, setEmpty], ["All items disabled", allDisabled, setAllDisabled], ["Tinted surface", tinted, setTinted]].map(([label, value, set]) =>
        <label key={String(label)}><input type="checkbox" checked={Boolean(value)} onChange={event => (set as React.Dispatch<React.SetStateAction<boolean>>)(event.target.checked)} /> {String(label)}</label>)}
    </div>
    <p className="text-sm">Open with primary press, Enter, Space or arrows. Navigate without changing the value; Enter selects, Escape dismisses and restores focus.</p>
    <div className="space-y-6" style={{padding:16, background:tinted ? "#3478f6" : "var(--background)"}}>
      {(reverse ? [1, 0] : [0, 1]).map(index => <section key={index} className="space-y-2">
        <label className="block text-sm font-medium">Value {index === 0 ? "A" : "B"}</label>
        {select(index)}
        <output className="block text-sm">{index === 0 ? "A" : "B"}: {values[index] || "None"} · Changes: {changes[index]}</output>
      </section>)}
      <section className="space-y-2"><p className="text-sm font-medium">Disabled trigger</p>{select(0,true)}</section>
    </div>
    <Button variant="outline" onClick={() => setOutside(n => n + 1)}>Outside action: {outside}</Button>
  </main>;
}

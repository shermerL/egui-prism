import * as React from "react";
import { Button } from "./ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

const longText = "This is a long plain-text hint. Move from the trigger into the hint and back without losing it. It is bounded by the viewport, contains no links or actions, and never replaces the trigger's accessible label. Extra detail wraps rather than widening the page.";

export function TooltipCases() {
  const [dark, setDark] = React.useState(false);
  const [disabled, setDisabled] = React.useState(false);
  const [hidden, setHidden] = React.useState(false);
  const [long, setLong] = React.useState(false);
  const [reverse, setReverse] = React.useState(false);
  const [delayed, setDelayed] = React.useState(false);
  const [counts, setCounts] = React.useState([0, 0]);
  const [outside, setOutside] = React.useState(0);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  const hint = (index: number, alwaysDisabled = false) => <Tooltip>
    <TooltipTrigger asChild>
      <Button variant="outline" className="w-48" disabled={disabled || alwaysDisabled} onClick={() => {
        if (!alwaysDisabled) setCounts(old => old.map((count, i) => i === index ? count + 1 : count));
      }}>{alwaysDisabled ? "Disabled trigger" : `Action ${index === 0 ? "A" : "B"}`}</Button>
    </TooltipTrigger>
    <TooltipContent>
      {long ? longText : "Add to library"}
    </TooltipContent>
  </Tooltip>;
  return <main style={{padding:32, minHeight:"100vh", background:"var(--background)", color:"var(--foreground)"}} className="space-y-6">
    <h1 className="text-xl font-semibold">shadcn Tooltip comparison</h1>
    <p className="text-sm">Pinned shadcn 8a35366 · default top / centered · provider delay 0ms</p>
    <Button variant="outline" onClick={() => setDark(!dark)}>{dark ? "Light" : "Dark"}</Button>
    <div className="flex flex-wrap gap-4 text-sm">
      {([ ["Disabled", disabled, setDisabled], ["Hide B", hidden, setHidden], ["Long text", long, setLong], ["Reverse", reverse, setReverse], ["500ms delay", delayed, setDelayed] ] as const).map(([label, value, set]) =>
        <label key={label}><input type="checkbox" checked={value} onChange={event => set(event.target.checked)} /> {label}</label>)}
    </div>
    <p className="text-sm">Hover or Tab. Move into the hint and back. Escape, click and blur dismiss without swallowing activation. Scroll an ancestor; hide and restore B while pending.</p>
    <TooltipProvider delayDuration={delayed ? 500 : 0}>
      <div className="space-y-6 pt-12">
        {(reverse ? [1, 0] : [0, 1]).filter(index => index !== 1 || !hidden).map(index => <section key={index} className="space-y-2">
          {hint(index)}
          <output className="block text-sm">{index === 0 ? "A" : "B"} activations: {counts[index]}</output>
        </section>)}
        {hint(0, true)}
      </div>
    </TooltipProvider>
    <Button variant="outline" onClick={() => setOutside(count => count + 1)}>Outside action: {outside}</Button>
    <div style={{height:400}} />
    <p className="text-sm">Scroll endpoint. Use the viewport edges and a narrow window for collision checks. No CSS zoom is applied.</p>
  </main>;
}

import * as React from "react";
import { Plus } from "lucide-react";
import { Button } from "./ui/button";

export function ButtonCases() {
  const [disabled, setDisabled] = React.useState(false);
  const [invalid, setInvalid] = React.useState(false);
  const [count, setCount] = React.useState(0);
  const activate = () => setCount(value => value + 1);
  return <section className="space-y-3" aria-label="Button parity cases">
    <h2 className="text-xl font-semibold">Prism buttons</h2>
    <p className="text-sm">Hover, press, or use Tab and Enter/Space. Theme and scale changes preserve action counts.</p>
    <div className="flex gap-4 text-sm">
      <label><input type="checkbox" checked={disabled} onChange={e => setDisabled(e.target.checked)} /> Disable buttons</label>
      <label><input type="checkbox" checked={invalid} onChange={e => setInvalid(e.target.checked)} /> Invalid buttons</label>
    </div>
    <p className="text-sm">Roles and sizes: XS / Small / Default / Large</p>
    {(["default", "secondary", "outline", "ghost", "destructive", "link"] as const).map((variant, index) => <div key={variant} className="flex flex-wrap items-center gap-2">
      {(["xs", "sm", "default", "lg"] as const).map(size => <Button key={size} variant={variant} size={size} disabled={disabled} aria-invalid={invalid || undefined} onClick={activate}>{["Primary", "Secondary", "Outline", "Ghost", "Destructive", "Link"][index]}</Button>)}
    </div>)}
    <div className="flex flex-wrap items-center gap-2">
      <Button disabled={disabled} onClick={activate}><Plus /> Add item</Button>
      {(["icon-xs", "icon-sm", "icon", "icon-lg"] as const).map(size => <Button key={size} size={size} aria-label="Add item" disabled={disabled} aria-invalid={invalid || undefined} onClick={activate}><Plus /></Button>)}
    </div>
    <div className="overflow-x-auto py-1"><Button variant="outline" disabled={disabled} onClick={activate}>A deliberately long action label that keeps its intrinsic width</Button></div>
    <output className="text-sm">Preview activations: {count}</output>
  </section>;
}

import * as React from "react";
import { Input } from "./ui/input";

export function InputCases() {
  const [dark, setDark] = React.useState(false);
  const [disabled, setDisabled] = React.useState(false);
  const [invalid, setInvalid] = React.useState(false);
  const [reversed, setReversed] = React.useState(false);
  const [narrow, setNarrow] = React.useState(false);
  const [values, setValues] = React.useState(["", ""]);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  return <main style={{ minHeight: "100vh", padding: 32, background: "var(--background)", color: "var(--foreground)" }}>
    <h1 className="text-xl font-semibold">Pinned shadcn text inputs</h1>
    <div className="my-4 flex flex-wrap gap-4 text-sm">
      {([
        ["Dark", dark, setDark], ["Disabled", disabled, setDisabled],
        ["Invalid", invalid, setInvalid], ["Reverse", reversed, setReversed],
        ["320px content", narrow, setNarrow],
      ] as const).map(([label, value, setter]) => <label key={label} className="flex items-center gap-2">
        <input type="checkbox" checked={value} onChange={event => setter(event.target.checked)} />{label}
      </label>)}
    </div>
    <section className="space-y-6" style={{ width: narrow ? 320 : 640, maxWidth: "100%" }}>
      {(reversed ? [1, 0] : [0, 1]).map(index => <div key={index} className="space-y-2">
        <label className="text-sm" htmlFor={`value-${index}`}>Value {index === 0 ? "A" : "B"}</label>
        <Input id={`value-${index}`} value={values[index]} disabled={disabled}
          placeholder="Type or paste here" aria-invalid={invalid || undefined}
          aria-describedby={invalid ? `error-${index}` : undefined}
          onChange={event => setValues(previous => previous.map((v, i) => i === index ? event.target.value : v))} />
        {invalid && <p id={`error-${index}`} className="text-xs text-destructive">Caller-provided error: this description may wrap, and does not modify the value.</p>}
        <output className="block text-xs text-muted-foreground">Characters: {[...values[index]].length}</output>
      </div>)}
    </section>
    <details className="mt-6 text-sm"><summary>Inspect Unicode values</summary>
      {values.map((value, index) => <pre key={index}>{index === 0 ? "A" : "B"}: {[...value].map(character => `\\u{${character.codePointAt(0)!.toString(16)}}`).join("")}</pre>)}
    </details>
  </main>;
}

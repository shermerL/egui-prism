import * as React from "react";
import { PlusIcon } from "lucide-react";
import { Button } from "./ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemMedia, ItemTitle } from "./ui/item";

export function ItemCases() {
  const [dark, setDark] = React.useState(false);
  const [small, setSmall] = React.useState(false);
  const [narrow, setNarrow] = React.useState(false);
  const [disabled, setDisabled] = React.useState(false);
  const [hidden, setHidden] = React.useState(false);
  const [reversed, setReversed] = React.useState(false);
  const [long, setLong] = React.useState(false);
  const [media, setMedia] = React.useState(true);
  const [counts, setCounts] = React.useState([[0, 0], [0, 0], [0, 0]]);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  const activate = (item: number, action: number) => setCounts(previous => previous.map((row, i) => row.map((count, j) => count + (i === item && j === action ? 1 : 0))));
  return <main style={{padding:32,background:"var(--background)",color:"var(--foreground)",minHeight:"100vh"}}>
    <h1>Items</h1>
    <div className="flex flex-wrap gap-4 my-4">
      <label><input type="checkbox" checked={dark} onChange={e=>setDark(e.target.checked)}/> Dark</label>
      <label><input type="checkbox" checked={small} onChange={e=>setSmall(e.target.checked)}/> Small</label>
      <label><input type="checkbox" checked={narrow} onChange={e=>setNarrow(e.target.checked)}/> 320pt content</label>
      <label><input type="checkbox" checked={disabled} onChange={e=>setDisabled(e.target.checked)}/> Disable actions</label>
      <label><input type="checkbox" checked={hidden} onChange={e=>setHidden(e.target.checked)}/> Hide items</label>
      <label><input type="checkbox" checked={reversed} onChange={e=>setReversed(e.target.checked)}/> Reverse items</label>
      <label><input type="checkbox" checked={long} onChange={e=>setLong(e.target.checked)}/> Long copy and action</label>
      <label><input type="checkbox" checked={media} onChange={e=>setMedia(e.target.checked)}/> Leading icon</label>
    </div>
    <div className="space-y-4" style={{width:narrow?320:640,maxWidth:"100%"}}>
      {!hidden && (reversed?[2,1,0]:[0,1,2]).map(i=><React.Fragment key={i}>
        <Item variant={(["default","outline","muted"] as const)[i]} size={small?"sm":"default"}>
          {media && <ItemMedia variant="icon"><PlusIcon/></ItemMedia>}
          <ItemContent>
            <ItemTitle>{["Default item","Outlined item","Muted item"][i]}</ItemTitle>
            <ItemDescription>{long?"A deliberately long description demonstrates the supported text layout, keeps the complete accessible name, and must not steal the independent actions' hit regions.":"Caller-owned content and independent actions."}</ItemDescription>
          </ItemContent>
          <ItemActions>
            <Button variant="outline" disabled={disabled} onClick={()=>activate(i,0)}>{long?"Download original archive":"Open"}</Button>
            <Button size="icon" disabled={disabled} aria-label="Add item" onClick={()=>activate(i,1)}><PlusIcon/></Button>
          </ItemActions>
        </Item>
        <p>Item {i}: Open {counts[i][0]}, Add {counts[i][1]}</p>
      </React.Fragment>)}
    </div>
    <p>Use actual browser zoom at 100% and 150%. This is the original pinned component, not an approximation of the Rust layout.</p>
  </main>;
}

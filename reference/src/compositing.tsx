import * as React from "react";
import { Button } from "./ui/button";

const backgrounds=["#ffffff","#18181b","#4a8fc6","#e6b45a"];
const opacities=[0,.25,.5,.75,1];
const image="data:image/svg+xml,"+encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><path fill="white" d="M0 0h8v16H0z"/><path fill="white" fill-opacity="0.5019607843137255" d="M8 0h8v16H8z"/></svg>');

export function Compositing() {
 const [opacity,setOpacity]=React.useState(.5);
 const [parent,setParent]=React.useState(1);
 const [clip,setClip]=React.useState(false);
 const [scale,setScale]=React.useState(1);
 return <main style={{padding:32,background:"white",color:"#18181b",fontFamily:"Geist, sans-serif",zoom:scale}}>
  <h1>CSS group compositing reference</h1>
  <p>Real CSS opacity with the pinned shadcn stylesheet and components.</p>
  <div style={{display:"flex",gap:16,margin:"16px 0"}}>
   <label>Group <input aria-label="Group opacity" type="range" min="0" max="1" step=".01" value={opacity} onChange={e=>setOpacity(+e.target.value)}/></label>
   <label>Parent <input aria-label="Parent opacity" type="range" min="0" max="1" step=".01" value={parent} onChange={e=>setParent(+e.target.value)}/></label>
   <label><input type="checkbox" checked={clip} onChange={e=>setClip(e.target.checked)}/> Clip right edge</label>
   <button onClick={()=>setScale(scale===1?1.5:1)}>{scale===1?"150%":"100%"}</button>
  </div>
  <div data-testid="pixel-matrix" style={{display:"grid",gridTemplateColumns:"repeat(5,80px)",gap:"8px"}}>
   {backgrounds.flatMap((background,row)=>opacities.map(opacity=><div key={`${row}-${opacity}`} data-row={row} data-opacity={opacity} style={{position:"relative",width:80,height:56,background}}>
    <div style={{position:"absolute",inset:8,background:"#18181b",opacity}}>
     <img src={image} width="32" height="16" style={{position:"absolute",left:8,top:8,width:32,height:16,imageRendering:"pixelated"}}/>
    </div>
   </div>))}
  </div>
  <div data-testid="decorated-group" style={{position:"relative",width:200,height:90,marginTop:16,background:"conic-gradient(#d2dcf0 25%, #96aac8 0 50%, #d2dcf0 0 75%, #96aac8 0) 0 0 / 32px 32px"}}>
   <div style={{position:"absolute",inset:0,opacity:parent}}>
    <div style={{position:"absolute",inset:0,clipPath:clip?"inset(0 60px 0 0)":undefined}}>
     <div style={{position:"absolute",left:24,top:24,width:140,height:36,borderRadius:6,background:"var(--primary)",color:"white",boxShadow:"0 2px 8px #0000005a, 0 0 0 3px var(--ring)",opacity}}>
      <img src={image} width="32" height="16" style={{position:"absolute",left:10,top:10,width:32,height:16}}/>
      <span style={{position:"absolute",left:52,top:9,fontSize:14,lineHeight:"normal"}}>Group</span>
     </div>
    </div>
   </div>
  </div>
  <h2 style={{marginTop:24}}>Pinned component opacity, all roles</h2>
  {backgrounds.map(background=><div key={background} style={{display:"flex",gap:12,padding:16,background}}>
   {(["default","secondary","outline","ghost","destructive","link"] as const).map(variant=><Button key={variant} variant={variant} disabled><img src={image} width="16" height="16"/>{variant}</Button>)}
  </div>)}
 </main>;
}

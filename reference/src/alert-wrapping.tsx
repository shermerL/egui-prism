import * as React from "react";
import { Alert, AlertDescription } from "./ui/alert";
import corpus from "../fixtures/alert-wrapping.tsv?raw";
import manifest from "../manifest.json";

const cases = corpus.trimEnd().split("\n").map(line => {
  const tab = line.indexOf("\t");
  return { id: line.slice(0, tab), text: line.slice(tab + 1) };
});
const widths = [448, 320, 160, 24];

// Read actual text-node Range geometry. Do not insert spans: anonymous grid
// items and inline shaping must remain those of the unmodified pinned source.
function measure(element: HTMLElement) {
  const node = element.firstChild;
  if (!node || node.nodeType !== Node.TEXT_NODE || element.childNodes.length !== 1)
    throw new Error("Expected the pinned description's single plain text child");
  const text = node.textContent!;
  const lines: { top: number; left: number; right: number; text: string }[] = [];
  const unpositioned: { offset: number; text: string }[] = [];
  const glyphRanges: { offset: number; text: string; rects: { x: number; y: number; width: number; height: number }[] }[] = [];
  let offset = 0;
  for (const character of text) {
    const range = document.createRange();
    range.setStart(node, offset);
    offset += character.length;
    range.setEnd(node, offset);
    const rects = range.getClientRects();
    glyphRanges.push({ offset: offset - character.length, text: character, rects: Array.from(rects, rect => ({ x: rect.x, y: rect.y, width: rect.width, height: rect.height })) });
    if (rects.length === 0) { unpositioned.push({ offset: offset - character.length, text: character }); continue; }
    const rect = range.getBoundingClientRect();
    // Collapsed trailing spaces can have zero width; retain their source text
    // in the range's line, and expose raw line strings instead of trimming.
    let line = lines.find(line => Math.abs(line.top - rect.top) < 0.5);
    if (!line) {
      line = { top: rect.top, left: rect.left, right: rect.right, text: "" };
      lines.push(line);
    }
    line.text += character;
    line.left = Math.min(line.left, rect.left);
    line.right = Math.max(line.right, rect.right);
  }
  const style = getComputedStyle(element);
  const parent = element.closest<HTMLElement>("[data-slot=alert]")!;
  const rect = element.getBoundingClientRect();
  const parentRect = parent.getBoundingClientRect();
  return {
    text, unpositioned, glyphRanges, descriptionWidth: rect.width, alertWidth: parentRect.width,
    alertHeight: parentRect.height, scrollWidth: element.scrollWidth,
    overflowX: style.overflowX, font: style.font, lineHeight: style.lineHeight,
    whiteSpace: style.whiteSpace, wordBreak: style.wordBreak,
    overflowWrap: style.overflowWrap, gridColumns: getComputedStyle(parent).gridTemplateColumns,
    lines: lines.map(line => ({ text: line.text, top: line.top - rect.top, width: line.right - line.left })),
  };
}

export function AlertWrapping() {
  const [result, setResult] = React.useState("NOT RUN: select Measure after fonts load.");
  const [dark, setDark] = React.useState(false);
  React.useLayoutEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    return () => document.documentElement.classList.remove("dark");
  }, [dark]);
  async function capture() {
    try {
      await document.fonts.load('14px "Geist"');
      await document.fonts.ready;
      if (![...document.fonts].some(face => face.family.replaceAll('"', "") === "Geist" && face.status === "loaded"))
        throw new Error("Geist font is not ready");
      const samples = Array.from(document.querySelectorAll<HTMLElement>("[data-wrap-case]")).map(wrapper => ({
        id: wrapper.dataset.wrapCase, width: Number(wrapper.dataset.width),
        ...measure(wrapper.querySelector<HTMLElement>("[data-slot=alert-description]")!),
      }));
      setResult(JSON.stringify({
        status: "MEASURED_NOT_ACCEPTED", userAgent: navigator.userAgent,
        reference: manifest.shadcn_commit, geistPackage: manifest.geist_package,
        viewport: { width: innerWidth, height: innerHeight, dpr: devicePixelRatio, visualScale: visualViewport?.scale },
        dark, samples,
        limitation: "Browser geometry only. Match native revision/fonts/zoom and inspect missing glyphs; no CSS parity, native window or AT pass implied.",
      }, null, 2));
    } catch (error) { setResult(JSON.stringify({ status: "ERROR", error: String(error) })); }
  }
  return <main style={{ padding: 32, background: "var(--background)", color: "var(--foreground)" }}>
    <h1>Alert wrapping measurement</h1>
    <p>Use actual browser zoom, not CSS transforms. These are description-only, no-icon samples.
      CJK needs separately verified matching font coverage. Record the tested repository SHA with this output.</p>
    <label><input type="checkbox" checked={dark} onChange={event => setDark(event.target.checked)}/> Dark theme</label>
    <button onClick={capture}>Measure loaded reference</button>
    {widths.flatMap(width => cases.map(sample =>
      <section key={width + sample.id} data-wrap-case={sample.id} data-width={width} style={{ marginTop: 24 }}>
        <p>{sample.id}, {width}px</p>
        <Alert style={{ width }}><AlertDescription>{sample.text}</AlertDescription></Alert>
      </section>
    ))}
    <h2>Measurement JSON, not an acceptance verdict</h2>
    <pre style={{ whiteSpace: "pre-wrap" }}>{result}</pre>
  </main>;
}

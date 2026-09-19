import {createRoot} from "react-dom/client";
import {act} from "react";
import {App} from "./App";
import {beforeEach,describe,expect,it,vi} from "vitest";

beforeEach(() => {
  const storage = () => {
    const values = new Map<string,string>();
    return {getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => values.set(key,value), removeItem: (key: string) => values.delete(key), clear: () => values.clear()};
  };
  vi.stubGlobal("localStorage",storage());
  vi.stubGlobal("sessionStorage",storage());
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT",true);
  localStorage.clear();sessionStorage.clear();
});
describe("App",()=>{
  it("renders an editable unauthenticated composer with a truthful connection action",()=>{
    const el=document.createElement("div");document.body.appendChild(el);const root=createRoot(el);
    act(()=>root.render(<App/>));
    expect(el.textContent).toMatch(/What should happen/);
    expect(el.textContent).toMatch(/Run bf to open an authenticated workbench/);
    expect(el.querySelector("textarea")?.disabled).toBe(false);
    act(()=>root.unmount());el.remove();
  });
  it("authenticated refresh treats empty lists as connected, not missing routes", async () => {
    sessionStorage.setItem("bf.session", "tok");
    vi.stubGlobal("fetch", vi.fn(async (input: RequestInfo | URL) => {
      const path = String(input);
      if (path.includes("/v3/events")) {
        return new Response("data: {\"cursor\":0}\n\n", {status: 200, headers: {"Content-Type": "text/event-stream"}});
      }
      if (path.includes("/v3/projects") || path.includes("/v3/drafts") || path.includes("/v3/work")) {
        return new Response(JSON.stringify({items: []}), {status: 200, headers: {"Content-Type": "application/json"}});
      }
      return new Response(JSON.stringify({error: "not used"}), {status: 200, headers: {"Content-Type": "application/json"}});
    }));
    const el = document.createElement("div");
    document.body.appendChild(el);
    const root = createRoot(el);
    await act(async () => {root.render(<App/>);});
    await act(async () => {await Promise.resolve();});
    expect(el.textContent).not.toMatch(/404/);
    expect(el.textContent).toMatch(/Work/);
    const urls = (fetch as ReturnType<typeof vi.fn>).mock.calls.map(c => String(c[0]));
    expect(urls.some(u => u.includes("/v3/projects"))).toBe(true);
    expect(urls.some(u => u.includes("/v3/events"))).toBe(true);
    await act(async () => {root.unmount();});
    el.remove();
  });
});

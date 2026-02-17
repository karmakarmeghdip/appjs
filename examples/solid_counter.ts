import { createEffect, createRoot, createSignal } from "npm:solid-js";
import * as appjs from "../packages/appjs-runtime/src/index.ts";
import { createAppJsRenderer } from "../packages/solid-renderer/src/index.ts";

type RendererHost = {
    createRoot(parentId?: string | null): unknown;
    createHostElement(tag: string): { nodeType: "element"; widgetId: string };
    setHostProperty(node: { nodeType: "element"; widgetId: string }, name: string, value: unknown, prev?: unknown): void;
    appendHostNode(
        parent: unknown,
        node: { nodeType: "element" | "text"; widgetId: string },
        anchor?: { nodeType: "element" | "text"; widgetId: string } | null
    ): void;
};

appjs.window.setTitle("AppJS Solid Renderer Demo");
appjs.window.resize(680, 420);
appjs.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
});

const renderer = createAppJsRenderer(appjs) as unknown as RendererHost;
const root = renderer.createRoot(null);

const column = renderer.createHostElement("column");
renderer.setHostProperty(column, "gap", 14);
renderer.setHostProperty(column, "crossAxisAlignment", "center");
renderer.appendHostNode(root, column);

const header = renderer.createHostElement("label");
renderer.setHostProperty(header, "text", "Solid + AppJS Custom Renderer");
renderer.setHostProperty(header, "fontSize", 24);
renderer.setHostProperty(header, "fontWeight", 700);
renderer.setHostProperty(header, "color", "#cdd6f4");
renderer.appendHostNode(column, header);

const countLabel = renderer.createHostElement("label");
renderer.setHostProperty(countLabel, "text", "Count: 0");
renderer.setHostProperty(countLabel, "fontSize", 42);
renderer.setHostProperty(countLabel, "fontWeight", 900);
renderer.setHostProperty(countLabel, "color", "#89b4fa");
renderer.appendHostNode(column, countLabel);

const buttonRow = renderer.createHostElement("row");
renderer.setHostProperty(buttonRow, "gap", 10);
renderer.appendHostNode(column, buttonRow);

const decButton = renderer.createHostElement("button");
renderer.setHostProperty(decButton, "text", "-");
renderer.appendHostNode(buttonRow, decButton);

const resetButton = renderer.createHostElement("button");
renderer.setHostProperty(resetButton, "text", "Reset");
renderer.appendHostNode(buttonRow, resetButton);

const incButton = renderer.createHostElement("button");
renderer.setHostProperty(incButton, "text", "+");
renderer.appendHostNode(buttonRow, incButton);

const note = renderer.createHostElement("label");
renderer.setHostProperty(note, "text", "Buttons are wired using Solid signals/effects.");
renderer.setHostProperty(note, "fontSize", 13);
renderer.setHostProperty(note, "color", "#a6adc8");
renderer.appendHostNode(column, note);

createRoot(() => {
    const [count, setCount] = createSignal<number>(0);

    renderer.setHostProperty(incButton, "onClick", () => setCount((value: number) => value + 1));
    renderer.setHostProperty(decButton, "onClick", () => setCount((value: number) => value - 1));
    renderer.setHostProperty(resetButton, "onClick", () => setCount(0));

    createEffect(() => {
        renderer.setHostProperty(countLabel, "text", `Count: ${count()}`);
    });
});

appjs.log.info("Solid renderer demo initialized");

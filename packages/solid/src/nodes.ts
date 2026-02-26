import { HostElement, HostText, VellumRuntime, VellumRoot, HostParent } from "./types";
import { DEFAULT_PARENT_ID } from "./constants";

export function createNodeBuilder(runtime: VellumRuntime) {
  let fallbackId = 0;

  function nextWidgetId(prefix: string): string {
    if (runtime.nextId) return runtime.nextId();
    fallbackId += 1;
    return `__solid_${prefix}_${fallbackId}`;
  }

  function buildElementNode(tag: string): HostElement {
    return {
      nodeType: "element",
      tag,
      widgetId: nextWidgetId("el"),
      props: Object.create(null) as Record<string, unknown>,
      handlers: new Map(),
      parent: null,
      firstChild: null,
      nextSibling: null,
      mounted: false,
    };
  }

  function buildTextNode(value: string): HostText {
    return {
      nodeType: "text",
      widgetId: nextWidgetId("text"),
      text: String(value),
      parent: null,
      firstChild: null,
      nextSibling: null,
      mounted: false,
    };
  }

  return { buildElementNode, buildTextNode };
}

export function createRoot(parentWidgetId: string | null = DEFAULT_PARENT_ID): VellumRoot {
  return {
    nodeType: "root",
    parent: null,
    firstChild: null,
    nextSibling: null,
    mounted: true,
    parentWidgetId,
  };
}

export function getParentWidgetId(parent: HostParent): string | null {
  if (parent.nodeType === "root") return parent.parentWidgetId;
  return parent.widgetId;
}

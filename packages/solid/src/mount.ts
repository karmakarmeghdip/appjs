import { HostNode, HostElement, VellumRuntime } from "./types";
import { collectInitialWidgetState, applyMountedProperty } from "./props";

export function createMountManager(
  runtime: VellumRuntime,
  widgetNodeById: Map<string, HostElement>
) {
  function mountNode(node: HostNode, parentWidgetId: string | null): void {
    if (node.mounted) return;

    if (node.nodeType === "text") {
      runtime.ui.createWidget(node.widgetId, "label", parentWidgetId, node.text, null);
      node.mounted = true;
      return;
    }

    const init = collectInitialWidgetState(node);
    runtime.ui.createWidget(node.widgetId, init.kind, parentWidgetId, init.text, init.style, init.params, init.data);
    node.mounted = true;
    widgetNodeById.set(node.widgetId, node);

    for (const [name, value] of Object.entries(node.props)) {
      applyMountedProperty(runtime, node, name, value);
    }
  }

  function mountSubtree(node: HostNode, parentWidgetId: string | null): void {
    mountNode(node, parentWidgetId);
    if (node.nodeType === "text") return;

    let child = node.firstChild;
    while (child) {
      mountSubtree(child, node.widgetId);
      child = child.nextSibling;
    }
  }

  function unmountSubtree(node: HostNode): void {
    const children: HostNode[] = [];
    let child = node.firstChild;
    while (child) {
      children.push(child);
      child = child.nextSibling;
    }

    for (let i = children.length - 1; i >= 0; i -= 1) {
      unmountSubtree(children[i]);
    }

    if (node.nodeType === "element") {
      widgetNodeById.delete(node.widgetId);
      node.handlers.clear();
    }

    if (node.mounted) {
      runtime.ui.removeWidget(node.widgetId);
      node.mounted = false;
    }

    node.parent = null;
    node.nextSibling = null;
    node.firstChild = null;
  }

  function clearElementChildren(element: HostElement): void {
    const children: HostNode[] = [];
    let child = element.firstChild;
    while (child) {
      children.push(child);
      child = child.nextSibling;
    }

    for (let i = children.length - 1; i >= 0; i -= 1) {
      unmountSubtree(children[i]);
    }

    element.firstChild = null;
  }

  return { mountNode, mountSubtree, unmountSubtree, clearElementChildren };
}

/// <reference path="./shim.d.ts" />
import { createRenderEffect, runWithOwner } from "solid-js/dist/solid.js";
import type { Owner } from "solid-js";
import { createRenderer } from "solid-js/universal";
import {
  VellumRenderer,
  VellumRuntime,
  VellumRoot,
  VellumHostElement,
  VellumHostText,
  HostNode,
  HostElement,
  HostText,
  HostParent,
  RenderOptions,
} from "./types";
import { DEFAULT_PARENT_ID } from "./constants";
import {
  isNullish,
  isVellumJsxNode,
  isHostNodeLike,
  normalizeChildrenArray,
  isReactiveAccessorProp,
  unlinkFromParent,
  linkIntoParent,
  resolveChildValue,
  hasDynamicChildren,
} from "./utils";

import { createEventManager } from "./events";
import { applyMountedProperty } from "./props";
import { createNodeBuilder, createRoot, getParentWidgetId } from "./nodes";
import { createMountManager } from "./mount";

export function createVellumRenderer(runtime: VellumRuntime): VellumRenderer {
  const widgetNodeById = new Map<string, HostElement>();
  const jsxNodeMap = new WeakMap<object, HostNode>();

  const eventManager = createEventManager(runtime, widgetNodeById);
  const nodeBuilder = createNodeBuilder(runtime);
  const mountManager = createMountManager(runtime, widgetNodeById);

  function setElementProperty(node: HostElement, name: string, value: unknown, prev: unknown): void {
    if (name === "ref" && typeof value === "function") {
      value(node);
      return;
    }

    if (name === "id" && typeof value === "string" && !node.mounted) {
      node.widgetId = value;
    }

    const hadProp = Object.prototype.hasOwnProperty.call(node.props, name);
    if (isNullish(value) || value === false) {
      if (hadProp) {
        delete node.props[name];
      }
    } else {
      node.props[name] = value;
    }

    if (eventManager.applyEventProperty(node, name, value, prev)) {
      return;
    }

    if (node.mounted) {
      applyMountedProperty(runtime, node, name, value);
    }
  }

  function insertHostNode(parent: HostParent, node: HostNode, anchor: HostNode | null = null): void {
    node.parent = parent;
    linkIntoParent(parent, node, anchor);

    if (parent.mounted) {
      mountManager.mountSubtree(node, getParentWidgetId(parent));
    }
  }

  function reconcileElementChildren(element: HostElement, childrenValue: unknown): void {
    mountManager.clearElementChildren(element);

    const children = normalizeChildrenArray(childrenValue);

    for (const child of children) {
      const hostChild = materializeHostNode(child);
      if (!hostChild) continue;
      insertHostNode(element, hostChild, null);
    }
  }

  function materializeHostNode(input: unknown): HostNode | null {
    const resolved = resolveChildValue(input);

    if (isHostNodeLike(resolved)) {
      return resolved;
    }

    if (typeof resolved === "string" || typeof resolved === "number") {
      return nodeBuilder.buildTextNode(String(resolved));
    }

    if (!isVellumJsxNode(resolved)) {
      return null;
    }

    const cached = jsxNodeMap.get(resolved as object);
    if (cached) {
      return cached;
    }

    const element = nodeBuilder.buildElementNode(resolved.type);
    jsxNodeMap.set(resolved as object, element);

    const props = resolved.props ?? {};
    for (const [key, value] of Object.entries(props)) {
      if (key === "children") continue;

      if (isReactiveAccessorProp(key, value)) {
        let prev: unknown = undefined;
        const setupEffect = () => {
          createRenderEffect(() => {
            const next = value();
            setElementProperty(element, key, next, prev);
            prev = next;
          });
        };

        if (resolved.owner) {
          runWithOwner(resolved.owner as Owner, setupEffect);
        } else {
          setupEffect();
        }
        continue;
      }

      setElementProperty(element, key, value, undefined);
    }

    const childrenProp = props.children;
    if (typeof childrenProp === "function" || hasDynamicChildren(childrenProp)) {
      const setupChildrenEffect = () => {
        createRenderEffect(() => {
          const resolvedChildren = typeof childrenProp === "function" ? childrenProp() : childrenProp;
          reconcileElementChildren(element, resolvedChildren);
        });
      };

      if (resolved.owner) {
        runWithOwner(
          resolved.owner as Owner,
          setupChildrenEffect
        );
      } else {
        setupChildrenEffect();
      }
    } else {
      reconcileElementChildren(element, childrenProp);
    }

    return element;
  }

  const renderer = createRenderer<HostNode | VellumRoot>({
    createElement(tag: string): HostElement {
      return nodeBuilder.buildElementNode(tag);
    },
    createTextNode(value: string): HostText {
      return nodeBuilder.buildTextNode(value);
    },
    replaceText(node: HostNode | VellumRoot, value: string): void {
      if (node.nodeType !== "text") return;

      node.text = String(value);
      if (node.mounted) {
        runtime.ui.setText(node.widgetId, node.text);
      }
    },
    setProperty(node: HostNode | VellumRoot, name: string, value: unknown, prev: unknown): void {
      if (node.nodeType !== "element") return;
      setElementProperty(node, name, value, prev);
    },
    insertNode(parent: HostNode | VellumRoot, node: HostNode | VellumRoot, anchor?: HostNode | VellumRoot): void {
      const hostParent = parent as HostParent;
      const hostNode = materializeHostNode(node);
      if (!hostNode) return;

      const hostAnchor = materializeHostNode(anchor) ?? null;
      insertHostNode(hostParent, hostNode, hostAnchor);
    },
    isTextNode(node: HostNode | VellumRoot): boolean {
      return node.nodeType === "text";
    },
    removeNode(parent: HostNode | VellumRoot, node: HostNode | VellumRoot): void {
      const hostParent = parent as HostParent;
      const hostNode = materializeHostNode(node);
      if (!hostNode) return;

      unlinkFromParent(hostParent, hostNode);
      mountManager.unmountSubtree(hostNode);
    },
    getParentNode(node: HostNode | VellumRoot): HostParent | undefined {
      return node.parent ?? undefined;
    },
    getFirstChild(node: HostNode | VellumRoot): HostNode | undefined {
      return node.firstChild ?? undefined;
    },
    getNextSibling(node: HostNode | VellumRoot): HostNode | undefined {
      return node.nextSibling ?? undefined;
    },
  });

  function render(code: () => unknown, options?: RenderOptions | VellumRoot): VellumRoot {
    let root: VellumRoot;
    if (options && "nodeType" in options) {
      root = options;
    } else {
      const renderOptions = options as RenderOptions | undefined;
      root = createRoot(renderOptions?.parentId ?? DEFAULT_PARENT_ID);
    }

    renderer.render(code as () => HostNode | VellumRoot, root);
    return root;
  }

  function dispose(): void {
    eventManager.dispose();
    widgetNodeById.clear();
  }

  return {
    ...renderer,
    createRoot,
    createHostElement: (tag: string): VellumHostElement => nodeBuilder.buildElementNode(tag),
    createHostText: (value: string): VellumHostText => nodeBuilder.buildTextNode(value),
    setHostProperty: (node: VellumHostElement, name: string, value: unknown, prev?: unknown): void => {
      setElementProperty(node as HostElement, name, value, prev);
    },
    appendHostNode: (
      parent: VellumRoot | VellumHostElement,
      node: VellumHostElement | VellumHostText,
      anchor?: VellumHostElement | VellumHostText | null
    ): void => {
      insertHostNode(parent as HostParent, node as HostNode, (anchor as HostNode | null | undefined) ?? null);
    },
    render,
    dispose,
  };
}

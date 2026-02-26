import { HostElement, VellumRuntime, WidgetActionHandler } from "./types";
import { EVENT_WILDCARD } from "./constants";
import { normalizeEventName } from "./utils";

export function createEventManager(runtime: VellumRuntime, widgetNodeById: Map<string, HostElement>) {
  let unsubscribeEvents: (() => void) | null = null;

  function ensureEventSubscription(): void {
    if (unsubscribeEvents) return;

    unsubscribeEvents = runtime.events.on(EVENT_WILDCARD, (event) => {
      const widgetId = event.widgetId;
      if (!widgetId) return;

      const node = widgetNodeById.get(widgetId);
      if (!node) return;

      const action = event.action ?? EVENT_WILDCARD;
      const specific = node.handlers.get(action);
      if (specific) {
        for (const handler of specific) handler(event);
      }

      const wildcard = node.handlers.get(EVENT_WILDCARD);
      if (wildcard) {
        for (const handler of wildcard) handler(event);
      }
    });
  }

  function applyEventProperty(
    node: HostElement,
    propName: string,
    value: unknown,
    prev: unknown
  ): boolean {
    const action = normalizeEventName(propName);
    if (!action) return false;

    const handlers = node.handlers.get(action) ?? new Set<WidgetActionHandler>();

    if (typeof prev === "function") {
      handlers.delete(prev as WidgetActionHandler);
    }

    if (typeof value === "function") {
      handlers.add(value as WidgetActionHandler);
    }

    if (handlers.size > 0) {
      node.handlers.set(action, handlers);
      ensureEventSubscription();
    } else {
      node.handlers.delete(action);
    }

    return true;
  }

  function dispose(): void {
    if (unsubscribeEvents) {
      unsubscribeEvents();
      unsubscribeEvents = null;
    }
  }

  return {
    applyEventProperty,
    dispose,
  };
}

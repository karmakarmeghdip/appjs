import { HostElement, VellumRuntime, VellumStyle } from "./types";
import {
  isEventProp,
  normalizeWidgetKind,
  mapStyleKey,
  isNullish,
  isPrimitiveStyleValue,
  createEmptyStyle,
} from "./utils";

export function collectInitialWidgetState(node: HostElement): {
  kind: string;
  text: string | null;
  style: VellumStyle | null;
  params: Record<string, unknown> | null;
  data: Uint8Array | null;
} {
  const kind = normalizeWidgetKind(node.tag);
  const style = createEmptyStyle();
  const params: Record<string, unknown> = Object.create(null);
  let hasStyle = false;
  let hasParams = false;
  let text: string | null = null;

  if (node.tag === "row") {
    style.direction = "row";
    hasStyle = true;
  } else if (node.tag === "column" || node.tag === "div" || node.tag === "section" || node.tag === "main" || node.tag === "article") {
    style.direction = "column";
    hasStyle = true;
  }

  for (const [name, value] of Object.entries(node.props)) {
    if (name === "children" || name === "ref" || name === "key") continue;
    if (isEventProp(name) || isNullish(value)) continue;
    if (name === "id") continue;
    if (name === "type") continue;
    if (name === "visible") continue;
    if (name === "data") continue;
    if (name === "objectFit") continue;
    if (name === "src" || name === "playing" || name === "position") continue;

    if (name === "text") {
      if (kind === "button") {
        throw new Error(
          "<button text={...}> is deprecated. Use <button><label text={...} /></button> instead."
        );
      }
      text = String(value);
      continue;
    }

    if (name === "checked") {
      if (kind === "checkbox") {
        params.checked = Boolean(value);
        hasParams = true;
      }
      continue;
    }

    if (name === "value" && typeof value === "number") {
      if (kind === "slider" || kind === "progressBar") {
        params.value = value;
        hasParams = true;
      }
      continue;
    }

    if (name === "min" && typeof value === "number" && kind === "slider") {
      params.minValue = value;
      hasParams = true;
      continue;
    }

    if (name === "max" && typeof value === "number" && kind === "slider") {
      params.maxValue = value;
      hasParams = true;
      continue;
    }

    if (name === "step" && typeof value === "number" && kind === "slider") {
      params.step = value;
      hasParams = true;
      continue;
    }

    if (name === "placeholder" && typeof value === "string" && kind === "textInput") {
      params.placeholder = value;
      hasParams = true;
      continue;
    }

    if (name === "style" && typeof value === "object") {
      Object.assign(style, value as VellumStyle);
      hasStyle = true;
      continue;
    }

    if (isPrimitiveStyleValue(value)) {
      style[mapStyleKey(name)] = value;
      hasStyle = true;
    }
  }

  // Extract image-specific props
  let data: Uint8Array | null = null;

  if (kind === "image") {
    const rawData = node.props.data;
    if (rawData instanceof Uint8Array) {
      data = rawData;
    }
    const objectFit = node.props.objectFit;
    if (typeof objectFit === "string") {
      params.object_fit = objectFit;
      hasParams = true;
    }
  }

  if (kind === "video") {
    const src = node.props.src;
    if (typeof src === "string") {
      params.src = src;
      hasParams = true;
    }
  }

  if (kind === "progressBar" && params.value !== undefined && params.progress === undefined) {
    params.progress = params.value;
    delete params.value;
  }

  return {
    kind,
    text,
    style: hasStyle ? style : null,
    params: hasParams ? params : null,
    data,
  };
}

export function applyMountedProperty(runtime: VellumRuntime, node: HostElement, name: string, value: unknown): void {
  if (name === "children" || name === "ref" || name === "key" || name === "id") return;
  if (name === "type" || name === "src") return;
  if (isEventProp(name)) return;

  if (name === "playing" && typeof value === "boolean") {
    if (value) {
      runtime.ui.playVideo?.(node.widgetId);
    } else {
      runtime.ui.pauseVideo?.(node.widgetId);
    }
    return;
  }

  if (name === "position" && typeof value === "number") {
    runtime.ui.seekVideo?.(node.widgetId, value);
    return;
  }

  if (name === "style") {
    if (value && typeof value === "object") {
      runtime.ui.setStyle(node.widgetId, value as VellumStyle);
    }
    return;
  }

  if (name === "text") {
    runtime.ui.setText(node.widgetId, String(value ?? ""));
    return;
  }

  if (name === "visible") {
    runtime.ui.setVisible(node.widgetId, Boolean(value));
    return;
  }

  if (name === "checked") {
    runtime.ui.setChecked(node.widgetId, Boolean(value));
    return;
  }

  if (name === "value" && typeof value === "number") {
    runtime.ui.setValue(node.widgetId, value);
    return;
  }

  if (name === "min" || name === "max" || name === "step" || name === "placeholder") {
    return;
  }

  if (isPrimitiveStyleValue(value)) {
    runtime.ui.setStyleProperty(node.widgetId, mapStyleKey(name), value);
  }

  if (name === "data" && value instanceof Uint8Array && runtime.ui.setImageData) {
    runtime.ui.setImageData(node.widgetId, value);
  }
}

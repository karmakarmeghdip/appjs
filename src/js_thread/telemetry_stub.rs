deno_core::extension!(
    appjs_telemetry_stub,
    esm = [
        "ext:deno_telemetry/telemetry.ts" = {
            source = r#"
export const TRACING_ENABLED = false;
export const METRICS_ENABLED = false;

export const builtinTracer = {
  startSpan() {
    return {
      setAttributes() {},
      recordException() {},
      setStatus() {},
      end() {},
    };
  },
};

export class ContextManager {
  static setGlobalContextManager() {}
}

export function currentSnapshot() {
  return null;
}

export const PROPAGATORS = {
  inject() {},
  extract() {
    return null;
  },
};

export function restoreSnapshot() {
  return null;
}

export function enterSpan(_span, callback) {
  if (typeof callback === "function") {
    return callback();
  }
  return undefined;
}
"#
        },
        "ext:deno_telemetry/util.ts" = {
            source = r#"
export function updateSpanFromError() {}
export function updateSpanFromRequest() {}
export function updateSpanFromResponse() {}
"#
        }
    ],
);

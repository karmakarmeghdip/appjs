// AppJS IPC Bridge -- JavaScript API
// Exposes globalThis.appjs for controlling the UI and listening for events
const core = globalThis.Deno.core;

// ============================================================
// Event emitter internals
// ============================================================
const _listeners = {};
let _eventLoopRunning = false;

function _dispatch(eventJson) {
    const event = JSON.parse(eventJson);
    const type = event.type;
    if (!type) return;

    const handlers = _listeners[type];
    if (handlers) {
        for (const handler of handlers) {
            try {
                handler(event);
            } catch (err) {
                console.error(`[appjs] Error in '${type}' handler:`, err);
            }
        }
    }

    // Also dispatch to wildcard listeners
    const wildcardHandlers = _listeners["*"];
    if (wildcardHandlers) {
        for (const handler of wildcardHandlers) {
            try {
                handler(event);
            } catch (err) {
                console.error("[appjs] Error in wildcard handler:", err);
            }
        }
    }
}

async function _startEventLoop() {
    if (_eventLoopRunning) return;
    _eventLoopRunning = true;

    while (_eventLoopRunning) {
        try {
            const eventJson = await core.ops.op_wait_for_event();
            if (!eventJson) {
                _eventLoopRunning = false;
                break;
            }

            const parsed = JSON.parse(eventJson);
            if (parsed.type === "disconnected") {
                _eventLoopRunning = false;
                break;
            }

            _dispatch(eventJson);
        } catch (err) {
            console.error("[appjs] Event loop error:", err);
            _eventLoopRunning = false;
            break;
        }
    }
}

// ============================================================
// Public API: globalThis.appjs
// ============================================================
globalThis.appjs = {
    // ---- Window management ----
    window: {
        setTitle: (title) => core.ops.op_set_title(title),
        resize: (width, height) => core.ops.op_resize_window(width, height),
        close: () => core.ops.op_close_window(),
    },

    // ---- UI / Widget management ----
    ui: {
        createWidget: (id, kind, parentId, text) =>
            core.ops.op_create_widget(id, kind, parentId ?? null, text ?? null),
        removeWidget: (id) => core.ops.op_remove_widget(id),
        setWidgetText: (id, text) => core.ops.op_set_widget_text(id, text),
        setWidgetVisible: (id, visible) => core.ops.op_set_widget_visible(id, visible),
    },

    // ---- Event system ----
    events: {
        /**
         * Register a listener for a UI event type.
         * Supported types: windowResized, mouseClick, mouseMove, keyPress,
         *   keyRelease, textInput, widgetAction, windowFocusChanged,
         *   windowCloseRequested, appExit
         * Use "*" to listen for all events.
         *
         * @param {string} type - Event type name
         * @param {function} callback - Handler function receiving the event object
         * @returns {function} unsubscribe function
         */
        on: (type, callback) => {
            if (!_listeners[type]) {
                _listeners[type] = [];
            }
            _listeners[type].push(callback);

            // Auto-start the event loop on first listener registration
            if (!_eventLoopRunning) {
                _startEventLoop();
            }

            // Return unsubscribe function
            return () => {
                const handlers = _listeners[type];
                if (handlers) {
                    const idx = handlers.indexOf(callback);
                    if (idx >= 0) handlers.splice(idx, 1);
                }
            };
        },

        /**
         * Remove all listeners for a specific event type, or all listeners.
         * @param {string} [type] - If provided, only remove listeners for this type
         */
        off: (type) => {
            if (type) {
                delete _listeners[type];
            } else {
                for (const key of Object.keys(_listeners)) {
                    delete _listeners[key];
                }
            }
        },
    },

    // ---- Logging ----
    log: {
        debug: (msg) => core.ops.op_log("debug", String(msg)),
        info: (msg) => core.ops.op_log("info", String(msg)),
        warn: (msg) => core.ops.op_log("warn", String(msg)),
        error: (msg) => core.ops.op_log("error", String(msg)),
    },

    // ---- App lifecycle ----
    exit: () => core.ops.op_exit_app(),
};

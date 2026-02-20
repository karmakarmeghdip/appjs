import process from "node:process";
import net from "node:net";
import os from "node:os";
import { decode, encode } from "@msgpack/msgpack";

const SOCKET_PATH = process.platform === "win32"
    ? `${os.tmpdir()}\\appjs.sock`
    : "/tmp/appjs.sock";

export type BridgeEvent = {
    type: string;
    widgetId?: string;
    action?: string;
    value?: string | number | boolean;
};

export type JsToRustMessage =
    | { type: "setTitle"; title: string }
    | {
        type: "createWidget";
        id: string;
        kind: string;
        parent_id: string | null;
        text: string | null;
        style_json: string | null;
    }
    | { type: "removeWidget"; id: string }
    | { type: "setWidgetText"; id: string; text: string }
    | { type: "setWidgetVisible"; id: string; visible: boolean }
    | { type: "setWidgetValue"; id: string; value: number }
    | { type: "setWidgetChecked"; id: string; checked: boolean }
    | { type: "setWidgetStyle"; id: string; style_json: string }
    | { type: "setStyleProperty"; id: string; property: string; value: string }
    | { type: "resizeWindow"; width: number; height: number }
    | { type: "closeWindow" }
    | { type: "log"; level: string; message: string }
    | { type: "exitApp" }
    | { type: "ready" };

type RustToJsMessage =
    | { type: "uiEvent"; event: unknown }
    | { type: "shutdown" };

export type Bridge = {
    send(message: JsToRustMessage): void;
    onEvent(callback: (event: BridgeEvent) => void): () => void;
};

type AppJsGlobal = typeof globalThis & {
    __APPJS_BRIDGE__?: Bridge;
    __APPJS_SOCKET__?: net.Socket;
    __APPJS_CONSOLE_PATCHED__?: boolean;
};

function writeFrame(socket: net.Socket, message: JsToRustMessage): void {
    const payload = Buffer.from(encode(message));
    const frame = Buffer.allocUnsafe(4 + payload.length);
    frame.writeUInt32LE(payload.length, 0);
    payload.copy(frame, 4);
    socket.write(frame);
}

function mapUiEvent(event: unknown): BridgeEvent {
    const widgetAction = (event as { WidgetAction?: { widget_id?: string; action?: unknown } })?.WidgetAction;
    if (!widgetAction) {
        return { type: "unknown" };
    }

    if (widgetAction.action === "Click") {
        return {
            type: "widgetAction",
            widgetId: widgetAction.widget_id,
            action: "click",
        };
    }

    const valueChanged = (widgetAction.action as { ValueChanged?: number } | undefined)?.ValueChanged;
    if (valueChanged !== undefined) {
        return {
            type: "widgetAction",
            widgetId: widgetAction.widget_id,
            action: "valueChanged",
            value: valueChanged,
        };
    }

    return { type: "unknown" };
}

function formatLogArgs(args: unknown[]): string {
    return args
        .map((arg) => {
            if (typeof arg === "string") return arg;
            if (typeof arg === "number" || typeof arg === "boolean" || typeof arg === "bigint") {
                return String(arg);
            }
            if (arg instanceof Error) {
                return `${arg.name}: ${arg.message}\n${arg.stack ?? ""}`;
            }
            try {
                return JSON.stringify(arg);
            } catch {
                return String(arg);
            }
        })
        .join(" ");
}

function patchConsole(bridge: Bridge): void {
    const globalScope = globalThis as AppJsGlobal;
    if (globalScope.__APPJS_CONSOLE_PATCHED__) return;
    globalScope.__APPJS_CONSOLE_PATCHED__ = true;

    const send = (level: "debug" | "info" | "warn" | "error", args: unknown[]) => {
        try {
            if (globalScope.__APPJS_SOCKET__ && !globalScope.__APPJS_SOCKET__.destroyed) {
                bridge.send({ type: "log", level, message: formatLogArgs(args) });
            } else {
                process.stderr.write(`${formatLogArgs(args)}\n`);
            }
        } catch {
            process.stderr.write(`${formatLogArgs(args)}\n`);
        }
    };

    console.log = (...args: unknown[]) => send("info", args);
    console.info = (...args: unknown[]) => send("info", args);
    console.debug = (...args: unknown[]) => send("debug", args);
    console.warn = (...args: unknown[]) => send("warn", args);
    console.error = (...args: unknown[]) => send("error", args);
}

export function initAppJsBridge(): Bridge {
    const globalScope = globalThis as AppJsGlobal;
    if (globalScope.__APPJS_BRIDGE__) {
        return globalScope.__APPJS_BRIDGE__;
    }

    const listeners = new Set<(event: BridgeEvent) => void>();
    let readBuffer = Buffer.alloc(0);
    const messageQueue: JsToRustMessage[] = [];
    let isConnected = false;
    let socket: net.Socket | null = null;

    const bridge: Bridge = {
        send(message) {
            if (isConnected && socket && !socket.destroyed) {
                writeFrame(socket, message);
            } else {
                messageQueue.push(message);
            }
        },
        onEvent(callback) {
            listeners.add(callback);
            return () => {
                listeners.delete(callback);
            };
        },
    };

    globalScope.__APPJS_BRIDGE__ = bridge;

    socket = net.createConnection(SOCKET_PATH, () => {
        isConnected = true;
        globalScope.__APPJS_SOCKET__ = socket!;
        patchConsole(bridge);
        bridge.send({ type: "ready" });

        for (const msg of messageQueue) {
            writeFrame(socket!, msg);
        }
        messageQueue.length = 0;
    });

    socket.on("error", (err) => {
        process.stderr.write(`[appjs bridge] Socket connection error: ${String(err)}\n`);
        process.exit(1);
    });

    const emitEvent = (event: BridgeEvent) => {
        for (const listener of listeners) {
            try {
                listener(event);
            } catch (err) {
                process.stderr.write(`[appjs bridge] Event listener error: ${String(err)}\n`);
            }
        }
    };

    const handleFrame = (frame: Buffer) => {
        try {
            const message = decode(frame) as RustToJsMessage;
            if (message?.type === "uiEvent") {
                emitEvent(mapUiEvent(message.event));
                return;
            }
            if (message?.type === "shutdown") {
                if (socket) socket.end();
                process.exit(0);
            }
        } catch (err) {
            process.stderr.write(`[appjs bridge] Decode error: ${String(err)}\n`);
        }
    };

    const processReadBuffer = () => {
        while (readBuffer.length >= 4) {
            const len = readBuffer.readUInt32LE(0);
            if (readBuffer.length < 4 + len) {
                return;
            }

            const frame = readBuffer.subarray(4, 4 + len);
            readBuffer = readBuffer.subarray(4 + len);
            handleFrame(frame);
        }
    };

    socket.on("data", (chunk) => {
        readBuffer = Buffer.concat([readBuffer, Buffer.from(chunk)]);
        processReadBuffer();
    });

    socket.on("end", () => {
        process.exit(0);
    });

    return bridge;
}

export function ensureBridge(): Bridge {
    return initAppJsBridge();
}

initAppJsBridge();

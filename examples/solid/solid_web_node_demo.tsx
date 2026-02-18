import { createSignal } from "solid-js";
import * as appjs from "@appjs/runtime";
import { createAppJsRenderer } from "@appjs/solid-renderer";

appjs.window.setTitle("AppJS Web + Node API Demo (Solid)");
appjs.window.resize(860, 620);
appjs.body.setStyle({
    background: "#1e1e2e",
    padding: 20,
});

const renderer = createAppJsRenderer(appjs);

type RuntimeWithNodeGlobals = {
    require?: (specifier: string) => any;
    process?: {
        platform?: string;
        arch?: string;
        versions?: Record<string, string | undefined>;
    };
    Buffer?: {
        from(input: string): { toString(encoding?: string): string; length: number };
    };
};

function Demo() {
    const [fetchResult, setFetchResult] = createSignal("pending");
    const [cryptoResult, setCryptoResult] = createSignal("pending");
    const [urlResult, setUrlResult] = createSignal("pending");
    const [nodeProcessResult, setNodeProcessResult] = createSignal("pending");
    const [nodeBufferResult, setNodeBufferResult] = createSignal("pending");
    const [nodeOsResult, setNodeOsResult] = createSignal("pending");
    const [nodeFsResult, setNodeFsResult] = createSignal("pending");
    const [runStatus, setRunStatus] = createSignal("not run yet");

    const runDiagnostics = async () => {
        setRunStatus("running...");

        try {
            const response = await fetch("data:application/json,%7B%22demo%22%3A%22appjs%22%2C%22ok%22%3Atrue%7D");
            const payload = await response.json() as { demo: string; ok: boolean };
            setFetchResult(`ok (demo=${payload.demo}, ok=${payload.ok})`);
        } catch (err) {
            setFetchResult(`error: ${String(err)}`);
        }

        try {
            const encoder = new TextEncoder();
            const input = encoder.encode("appjs-web-crypto-demo");
            const digest = await crypto.subtle.digest("SHA-256", input);
            const bytes = new Uint8Array(digest);
            const preview = Array.from(bytes.slice(0, 6)).map((n) => n.toString(16).padStart(2, "0")).join("");
            const uuid = crypto.randomUUID();
            setCryptoResult(`ok (sha256[0..6]=${preview}, uuid=${uuid.slice(0, 8)}...)`);
        } catch (err) {
            setCryptoResult(`error: ${String(err)}`);
        }

        try {
            const query = new URLSearchParams({ source: "appjs", mode: "demo" });
            const url = new URL(`https://example.com/api?${query.toString()}`);
            setUrlResult(`ok (${url.hostname}${url.pathname}, source=${url.searchParams.get("source")})`);
        } catch (err) {
            setUrlResult(`error: ${String(err)}`);
        }

        const runtime = globalThis as unknown as RuntimeWithNodeGlobals;
        const req = runtime.require;

        try {
            if (!runtime.process) {
                throw new Error("process global missing");
            }
            setNodeProcessResult(
                `ok (platform=${runtime.process.platform}, arch=${runtime.process.arch}, node=${runtime.process.versions?.node ?? "n/a"})`,
            );
        } catch (err) {
            setNodeProcessResult(`error: ${String(err)}`);
        }

        try {
            if (!runtime.Buffer) {
                throw new Error("Buffer global missing");
            }
            const value = runtime.Buffer.from("AppJS desktop runtime");
            setNodeBufferResult(`ok (bytes=${value.length}, hex-prefix=${value.toString("hex").slice(0, 12)}...)`);
        } catch (err) {
            setNodeBufferResult(`error: ${String(err)}`);
        }

        try {
            if (!req) {
                throw new Error("require global missing");
            }
            const os = req("node:os") as { hostname(): string; cpus(): Array<unknown> };
            setNodeOsResult(`ok (hostname=${os.hostname()}, cpus=${os.cpus().length})`);
        } catch (err) {
            setNodeOsResult(`error: ${String(err)}`);
        }

        try {
            if (!req) {
                throw new Error("require global missing");
            }
            const fs = req("node:fs") as { existsSync(path: string): boolean; mkdirSync(path: string, opts: { recursive: boolean }): void };
            const path = req("node:path") as { join(...parts: string[]): string };
            const demoDir = path.join("/tmp", "appjs", "examples");
            fs.mkdirSync(demoDir, { recursive: true });
            setNodeFsResult(`ok (created=${demoDir}, exists=${fs.existsSync(demoDir)})`);
        } catch (err) {
            setNodeFsResult(`error: ${String(err)}`);
        }

        setRunStatus(`done @ ${new Date().toISOString()}`);
    };

    void runDiagnostics();

    return (
        <column gap={10}>
            <label text="SolidJS Demo: Web + Node APIs" fontSize={26} fontWeight={800} color="#cdd6f4" />
            <label text="Validates fetch, crypto, URL APIs, and Node globals/modules in AppJS runtime." color="#a6adc8" />

            <row gap={8}>
                <button text="Run Diagnostics" onClick={() => void runDiagnostics()} />
                <button
                    text="Clear"
                    onClick={() => {
                        setFetchResult("pending");
                        setCryptoResult("pending");
                        setUrlResult("pending");
                        setNodeProcessResult("pending");
                        setNodeBufferResult("pending");
                        setNodeOsResult("pending");
                        setNodeFsResult("pending");
                        setRunStatus("cleared");
                    }}
                />
            </row>

            <label text={() => `run status: ${runStatus()}`} color="#f9e2af" />

            <label text={() => `fetch: ${fetchResult()}`} color="#89b4fa" />
            <label text={() => `web crypto: ${cryptoResult()}`} color="#89dceb" />
            <label text={() => `url/urlsearchparams: ${urlResult()}`} color="#94e2d5" />
            <label text={() => `node process: ${nodeProcessResult()}`} color="#a6e3a1" />
            <label text={() => `node buffer: ${nodeBufferResult()}`} color="#fab387" />
            <label text={() => `node os module: ${nodeOsResult()}`} color="#f5c2e7" />
            <label text={() => `node fs/path modules: ${nodeFsResult()}`} color="#f38ba8" />
        </column>
    );
}

renderer.render(() => <Demo />);
appjs.log.info("Solid Web+Node API demo initialized");

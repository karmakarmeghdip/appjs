deno_core::extension!(
    appjs_web_bootstrap,
    esm_entry_point = "ext:appjs_web_bootstrap/runtime.js",
    esm = ["ext:appjs_web_bootstrap/runtime.js" = {
        source = r#"
import * as web from "ext:deno_web/04_global_interfaces.js";
import "ext:deno_web/01_broadcast_channel.js";
import "ext:deno_web/13_message_port.js";
import * as base64 from "ext:deno_web/05_base64.js";
import "ext:deno_web/16_image_data.js";
import "ext:deno_web/10_filereader.js";
import "ext:deno_web/14_compression.js";
import "ext:deno_web/15_performance.js";
import "ext:deno_web/01_urlpattern.js";
import * as file from "ext:deno_web/09_file.js";
import * as encoding from "ext:deno_web/08_text_encoding.js";
import * as abort from "ext:deno_web/03_abort_signal.js";
import * as url from "ext:deno_web/00_url.js";
import * as streams from "ext:deno_web/06_streams.js";
import * as headers from "ext:deno_fetch/20_headers.js";
import * as formData from "ext:deno_fetch/21_formdata.js";
import * as request from "ext:deno_fetch/23_request.js";
import * as response from "ext:deno_fetch/23_response.js";
import * as fetchMod from "ext:deno_fetch/26_fetch.js";
import "ext:deno_fetch/27_eventsource.js";
import * as cryptoMod from "ext:deno_crypto/00_crypto.js";
import process from "node:process";
import { Buffer } from "node:buffer";

if (globalThis.AbortController === undefined) globalThis.AbortController = abort.AbortController;
if (globalThis.AbortSignal === undefined) globalThis.AbortSignal = abort.AbortSignal;
if (globalThis.Blob === undefined) globalThis.Blob = file.Blob;
if (globalThis.File === undefined) globalThis.File = file.File;
if (globalThis.Headers === undefined) globalThis.Headers = headers.Headers;
if (globalThis.FormData === undefined) globalThis.FormData = formData.FormData;
if (globalThis.Request === undefined) globalThis.Request = request.Request;
if (globalThis.Response === undefined) globalThis.Response = response.Response;
if (globalThis.TextEncoder === undefined) globalThis.TextEncoder = encoding.TextEncoder;
if (globalThis.TextDecoder === undefined) globalThis.TextDecoder = encoding.TextDecoder;
if (globalThis.URL === undefined) globalThis.URL = url.URL;
if (globalThis.URLSearchParams === undefined) globalThis.URLSearchParams = url.URLSearchParams;
if (globalThis.ReadableStream === undefined) globalThis.ReadableStream = streams.ReadableStream;
if (globalThis.WritableStream === undefined) globalThis.WritableStream = streams.WritableStream;
if (globalThis.TransformStream === undefined) globalThis.TransformStream = streams.TransformStream;
if (globalThis.DOMException === undefined) globalThis.DOMException = web.DOMException;
if (globalThis.atob === undefined) globalThis.atob = base64.atob;
if (globalThis.btoa === undefined) globalThis.btoa = base64.btoa;

if (globalThis.fetch === undefined) globalThis.fetch = fetchMod.fetch;
if (globalThis.crypto === undefined) globalThis.crypto = cryptoMod.crypto;
if (globalThis.Crypto === undefined) globalThis.Crypto = cryptoMod.Crypto;
if (globalThis.CryptoKey === undefined) globalThis.CryptoKey = cryptoMod.CryptoKey;
if (globalThis.SubtleCrypto === undefined) globalThis.SubtleCrypto = cryptoMod.SubtleCrypto;
if (globalThis.process === undefined) globalThis.process = process;
if (globalThis.Buffer === undefined) globalThis.Buffer = Buffer;
"#
    }],
);

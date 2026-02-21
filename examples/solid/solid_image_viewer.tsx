// Image Viewer — Solid TSX version
// Fetches random images from picsum.photos
import * as appjs from "@appjs/runtime";
import { createAppJsRenderer, createSignal } from "@appjs/solid-renderer";

const WIDTH = 400;
const HEIGHT = 300;

async function fetchImageBytes(): Promise<Uint8Array> {
  const url = `https://picsum.photos/${WIDTH}/${HEIGHT}?t=${Date.now()}`;
  const response = await fetch(url, { redirect: "follow" });
  return new Uint8Array(await response.arrayBuffer());
}

appjs.window.setTitle("Image Viewer");
appjs.window.resize(500, 500);
appjs.body.setStyle({ background: "#1e1e2e", padding: 24 });

const renderer = createAppJsRenderer(appjs);

// Pre-fetch the first image before rendering
const initialData = await fetchImageBytes();

function ImageViewer() {
  const [status, setStatus] = createSignal(`Loaded (${initialData.byteLength} bytes)`);

  async function refetch() {
    setStatus("Fetching image...");
    try {
      const data = await fetchImageBytes();
      appjs.ui.setImageData("img", data);
      setStatus(`Loaded (${data.byteLength} bytes)`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  }

  return (
    <column gap={16} crossAxisAlignment="center">
      <label
        text="Random Image Viewer"
        fontSize={28}
        fontWeight={700}
        color="#cdd6f4"
      />
      <label
        text={() => status()}
        fontSize={14}
        color="#a6adc8"
      />
      <image
        id="img"
        data={initialData}
        objectFit="contain"
        width={WIDTH}
        height={HEIGHT}
      />
      <button onClick={() => refetch()}>
        <label text="🔄  New Image" color="white" />
      </button>
    </column>
  );
}

renderer.render(() => <ImageViewer />);
appjs.log.info("Solid image viewer initialized");

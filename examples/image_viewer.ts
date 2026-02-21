// Image Viewer Example
// Fetches a random image from picsum.photos, shows a spinner while loading,
// and provides a button to refetch a new image.
import * as appjs from "@appjs/runtime";
import type { AppJsEvent } from "@appjs/runtime";

const WIDTH = 400;
const HEIGHT = 300;

appjs.window.setTitle("Image Viewer");
appjs.body.setStyle({ background: "#1e1e2e", padding: 24 });

appjs.column("root", null, {
  gap: 16,
  crossAxisAlignment: "center",
});

appjs.label("header", "root", "Random Image Viewer", {
  fontSize: 28,
  fontWeight: 700,
  color: "#cdd6f4",
});

appjs.label("status", "root", "Loading...", {
  fontSize: 14,
  color: "#a6adc8",
});

// Container for the image / spinner area (must be Flex so removeWidget works)
appjs.column("imageArea", "root", {
  width: WIDTH,
  height: HEIGHT,
  crossAxisAlignment: "center",
});

appjs.button("fetchBtn", "root", "🔄  New Image");

let imageCreated = false;
let spinnerShown = false;

function showSpinner(): void {
  if (!spinnerShown) {
    appjs.spinner("loadingSpinner", "imageArea");
    spinnerShown = true;
  }
}

function hideSpinner(): void {
  if (spinnerShown) {
    appjs.ui.removeWidget("loadingSpinner");
    spinnerShown = false;
  }
}

// Show spinner initially
showSpinner();

async function fetchImage(): Promise<void> {
  showSpinner();
  appjs.ui.setText("status", "Fetching image...");

  try {
    // picsum.photos redirects to a random image each time
    const url = `https://picsum.photos/${WIDTH}/${HEIGHT}?t=${Date.now()}`;
    const response = await fetch(url, { redirect: "follow" });
    const buffer = await response.arrayBuffer();
    const data = new Uint8Array(buffer);

    hideSpinner();

    if (!imageCreated) {
      appjs.image("img", "imageArea", data, {
        objectFit: "contain",
        width: WIDTH,
        height: HEIGHT,
      });
      imageCreated = true;
    } else {
      appjs.ui.setImageData("img", data);
    }

    appjs.ui.setText("status", `Image loaded (${data.byteLength} bytes)`);
  } catch (err) {
    hideSpinner();
    appjs.ui.setText("status", `Error: ${err}`);
  }
}

appjs.events.on("widgetAction", (e: AppJsEvent) => {
  if (e.widgetId === "fetchBtn") {
    fetchImage();
  }
});

// Fetch the first image on startup
fetchImage();

appjs.log.info("Image viewer app initialized!");

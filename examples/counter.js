// Counter App Example

// Set window title
appjs.window.setTitle("Counter App");

// Structure:
// - Title Label
// - Counter Label
// - Buttons (Flex container?) - wait, no horizontal layout easy, so vertical stack is fine.

appjs.ui.createWidget("header", "Label", null, "Simple Counter App");

appjs.ui.createWidget("countLabel", "Label", null, "Count: 0");

appjs.ui.createWidget("incBtn", "Button", null, "Increment");
appjs.ui.createWidget("decBtn", "Button", null, "Decrement");

let count = 0;

function updateCount() {
    appjs.ui.setWidgetText("countLabel", `Count: ${count}`);
}

appjs.events.on("widgetAction", (e) => {
    // Log the event for debugging
    appjs.log.info(`Action on ${e.widgetId}: ${e.action}`);

    if (e.widgetId === "incBtn") {
        count++;
        updateCount();
    } else if (e.widgetId === "decBtn") {
        count--;
        updateCount();
    }
});

appjs.log.info("Counter app initialized. Waiting for clicks...");

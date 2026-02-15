// Test script demonstrating the appjs IPC bridge API

// Window management
appjs.window.setTitle("Hello from JavaScript!");
console.log("Set window title");

// Create some widgets
appjs.ui.createWidget("header", "Label");
appjs.ui.setWidgetText("header", "Welcome to AppJS!");
console.log("Created header label");

appjs.ui.createWidget("btn1", "Button");
appjs.ui.setWidgetText("btn1", "Click Me!");
console.log("Created button");

appjs.ui.createWidget("input1", "TextInput");
console.log("Created text input");

// Test nested widget (with parent)
appjs.ui.createWidget("child-label", "Label", "header");
appjs.ui.setWidgetText("child-label", "I am a child widget");
console.log("Created child label");

// Test widget visibility
appjs.ui.setWidgetVisible("input1", false);
console.log("Hid input widget");
appjs.ui.setWidgetVisible("input1", true);
console.log("Showed input widget again");

// Test window resize
appjs.window.resize(1024, 768);
console.log("Resized window to 1024x768");

// Test logging via IPC
appjs.log.info("This is an info log via IPC");
appjs.log.warn("This is a warning via IPC");
appjs.log.debug("This is a debug log via IPC");

// Register event listeners
const unsubResize = appjs.events.on("windowResized", (e) => {
    console.log("Window resized to:", e.width, "x", e.height);
});

appjs.events.on("widgetAction", (e) => {
    console.log("Widget action:", e.widgetId, e.action);
});

appjs.events.on("mouseClick", (e) => {
    console.log("Mouse clicked at:", e.x, e.y);
});

// Wildcard listener
appjs.events.on("*", (e) => {
    console.log("Any event:", e.type);
});

console.log("All APIs tested! Event listeners registered.");
console.log("Waiting for UI events... (close the window to exit)");

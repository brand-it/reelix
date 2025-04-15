import { listen } from "@tauri-apps/api/event";
const unlisten = await listen("disks-changed", (event) => {
  console.log("disks-changed", event);
  processTurboResponse(event.payload);
});

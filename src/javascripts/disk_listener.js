import { listen } from "@tauri-apps/api/event";
const unlisten = await listen("disks-changed", (event) => {
  processTurboResponse(event.payload);
});

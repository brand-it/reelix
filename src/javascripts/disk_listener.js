import { listen } from "@tauri-apps/api/event";
console.log("disk_listener");
const unlisten = await listen("disks-changed", (event) => {
  console.log("disks-changed", event);
  processTurboResponse(event.payload);
});

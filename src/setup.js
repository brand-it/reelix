const originalFetch = window.fetch;
function uuid() {
  return Array.from({ length: 36 })
    .map((_, i) => {
      if (i == 8 || i == 13 || i == 18 || i == 23) {
        return "-";
      } else if (i == 14) {
        return "4";
      } else if (i == 19) {
        return (Math.floor(Math.random() * 4) + 8).toString(16);
      } else {
        return Math.floor(Math.random() * 15).toString(16);
      }
    })
    .join("");
}

async function turboInvoke(command, commandArgs) {
  console.log("turboInvoke", command, commandArgs);
  const tauriResponse = await window.__TAURI__.core
    .invoke(command, commandArgs)
    .catch((error) => {
      console.log("error", command, commandArgs, error);
      if (error.message == undefined) {
        document.getElementById("error").innerHTML = error;
      } else {
        document.getElementById("error").innerHTML = error.message;
      }
    });

  const parser = new DOMParser();
  const doc = parser.parseFromString(tauriResponse, "text/html");
  console.log("tauriResponse", tauriResponse);
  doc.querySelectorAll("turbo-stream").forEach((stream) => {
    console.log("renderStreamMessage", stream);
    Turbo.renderStreamMessage(stream.outerHTML);
  });
  return new Response(tauriResponse, {
    status: 200,
  });
}

document.addEventListener("DOMContentLoaded", (event) => {
  turboInvoke("index");
});

window.fetch = async (url, options = {}) => {
  let parsedUrl = new URL(url);
  // TODO: Improve to better ID what requests are Tauri & localhost requests
  if (parsedUrl.protocol == "http:" || parsedUrl.protocol == "tauri:") {
    let command = new URL(url).pathname.replace("/", "");
    let commandArgs = Object.fromEntries(options.body.entries());
    // Create a new Response object to mimic a real fetch response
    // TODO: I need to make this more turbo like, kinda of sucks I can't get access to the original Response object.
    return turboInvoke(command, commandArgs);
  } else {
    return originalFetch(url, {
      ...options,
    });
  }
};

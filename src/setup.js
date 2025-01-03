const originalFetch = window.fetch;

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

document.addEventListener("click", function (event) {
  if (event.target.tagName === "A" && event.target.href != undefined) {
    event.preventDefault();
    turboInvoke("open_browser", { url: event.target.href });
  }
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

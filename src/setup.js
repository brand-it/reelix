const originalFetch = window.fetch;

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

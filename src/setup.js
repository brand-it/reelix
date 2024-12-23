const originalFetch = window.fetch;
console.log("Overwrite originFetch");
function uuid() {
  return Array.from({ length: 36 })
    .map((_, i) => {
      if (i == 8 || i == 13 || i == 18 || i == 23) {
        return "-"
      } else if (i == 14) {
        return "4"
      } else if (i == 19) {
        return (Math.floor(Math.random() * 4) + 8).toString(16)
      } else {
        return Math.floor(Math.random() * 15).toString(16)
      }
    })
    .join("")
}


window.fetch = async (url, options = {}) => {
    let parsedUrl = new URL(url);
    // TODO: Improve to better ID what requests are Tauri & localhost requests
    if (parsedUrl.protocol == 'http:' || parsedUrl.protocol == 'tauri:') {
      let command = new URL(url).pathname.replace("/", "");
      let commandArgs = Object.fromEntries(options.body.entries())
      const tauriResponse = await window.__TAURI__.core.invoke(command, commandArgs).catch((err) => {
        document.getElementById('error').innerHTML = err.message
      });

      console.log('Intercepted fetch call:', url, command, commandArgs);


      // Create a new Response object to mimic a real fetch response
      // TODO: I need to make this more turbo like, kinda of sucks I can't get access to the original Response object.
      const parser = new DOMParser();
      const doc = parser.parseFromString(tauriResponse, 'text/html');
      doc.querySelectorAll('turbo-stream').forEach((stream) => {
        Turbo.renderStreamMessage(stream.outerHTML);
      });
      return new Response(tauriResponse, {
        status: 200,
      });
    } else {
      return originalFetch(url, {
        ...options
      })
    }
};

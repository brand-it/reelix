const originalFetch = window.fetch;
console.log("Overwrite originFetch");


window.fetch = async (url, options = {}) => {
    let parsedUrl = new URL(url);
    // TODO: Improve to better ID what requests are Tauri & localhost requests
    if (parsedUrl.protocol == 'http:') {
      let command = new URL(url).pathname.replace("/", "");
      let commandArgs = Object.fromEntries(options.body.entries())
      console.log('Intercepted fetch call:', url, command, commandArgs);
    const tauriResponse = await window.__TAURI__.core.invoke(command, commandArgs);

    // Create a new Response object to mimic a real fetch response
    // TODO: I need to make this more turbo like, kinda of sucks I can't get access to the original Response object.
    return new Response(JSON.stringify(tauriResponse), {
      headers: { 'Content-Type': 'application/json' },
      status: 200,
    });
    } else {
      return originalFetch(url, {
        ...options
      })
    }
};

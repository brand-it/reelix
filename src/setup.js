const originalFetch = window.fetch;
console.log("Overwrite originFetch");


window.fetch = async (url, options = {}) => {
    let parsedUrl = new URL(url);
    // TODO: Improve to better ID what requests are Tauri & localhost requests
    if (parsedUrl.protocol == 'http:') {
      let command = new URL(url).pathname.replace("/", "");
      console.log('Intercepted fetch call:', url, command);
      return await window.__TAURI__.core.invoke(command);
    } else {
      return originalFetch(url, {
        ...options
      })
    }
};

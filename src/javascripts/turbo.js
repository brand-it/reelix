import * as Turbo from "@hotwired/turbo"


// document.addEventListener("turbo:before-fetch-request", async (event) => {
//     const { fetchOptions } = event.detail;

//     // Prevent the default network request
//     event.preventDefault();

//     // Use IPC to handle the request
//     const response = await window.ipc.invoke("fetch", {
//         url: fetchOptions.url,
//         method: fetchOptions.method,
//         headers: fetchOptions.headers,
//         body: fetchOptions.body,
//     });

//     // Manually update the Turbo frame with the IPC response
//     const frame = document.querySelector("turbo-frame");
//     if (frame) {
//         frame.innerHTML = response.body;
//     }
// });


function addHasValue(input) {
  if (input.value.trim() !== '') {
    input.classList.add('has-val');
  } else {
    input.classList.remove('has-val');
  }
}

function initializeMinimalForm() {
  document.querySelectorAll('.minimal-input input').forEach(function (input) {
    addHasValue(input);
    input.addEventListener('blur', function () {
      addHasValue(this);
    });
  });
}

// Listen to Turbo events
document.addEventListener("turbo:load", initializeMinimalForm);
document.addEventListener("turbo:frame-load", initializeMinimalForm);
document.addEventListener("turbo:render", initializeMinimalForm);
document.addEventListener("turbo:before-render", initializeMinimalForm);
document.addEventListener("turbo:before-cache", initializeMinimalForm);

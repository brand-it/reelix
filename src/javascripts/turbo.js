import * as Turbo from "@hotwired/turbo"

// window.__TAURI_INTERNALS__.transformCallback = function(event, n = !1) {
//   console.log("event: transformCallback", event, n);
//     const t = window.crypto.getRandomValues(new Uint32Array(1))[0],
//         i = `_${t}`;
//     return Object.defineProperty(window, i, {
//         value: t => (n && Reflect.deleteProperty(window, i), event && event(t)),
//         writable: !1,
//         configurable: !0
//     }), t
// }
// document.addEventListener('turbo:before-fetch-request', (event) => {
//     const { fetchOptions, url } = event.detail;
//     console.log('Intercepting Turbo request:', url, fetchOptions);
//     fetchOptions.method = 'POST';

//     // Modify the protocol for specific URLs
//     if (url.protocol === 'http:') {
//         console.log("changing protocol from http: to ipc:")
//         event.detail.url = new URL(url.href.replace(/^http:/, 'ipc:'));
//     }
// });

// function addHasValue(input) {
//   if (input.value.trim() !== '') {
//     input.classList.add('has-val');
//   } else {
//     input.classList.remove('has-val');
//   }
// }

// function initializeMinimalForm() {
//   document.querySelectorAll('.minimal-input input').forEach(function (input) {
//     addHasValue(input);
//     input.addEventListener('blur', function () {
//       addHasValue(this);
//     });
//   });
// }

// // Listen to Turbo events
// document.addEventListener("turbo:load", initializeMinimalForm);
// document.addEventListener("turbo:frame-load", initializeMinimalForm);
// document.addEventListener("turbo:render", initializeMinimalForm);
// document.addEventListener("turbo:before-render", initializeMinimalForm);
// document.addEventListener("turbo:before-cache", initializeMinimalForm);

import * as Turbo from "@hotwired/turbo";

function splitPath(location) {
  return location.pathname.split("/").filter((element) => element !== "");
}

window.turboInvoke = async function turboInvoke(command, commandArgs) {
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
  console.log("tauriResponse", command, tauriResponse);
  processTurboResponse(tauriResponse);
  return new Response(tauriResponse, {
    status: 200,
  });
};

window.processTurboResponse = function processTurboResponse(turboResponse) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(turboResponse, "text/html");
  doc.querySelectorAll("turbo-stream").forEach((stream) => {
    Turbo.renderStreamMessage(stream.outerHTML);
  });
};

function findClosestRecursively(element, selector) {
  if (element instanceof Element) {
    return (
      element.closest(selector) ||
      findClosestRecursively(
        element.assignedSlot || element.getRootNode()?.host,
        selector
      )
    );
  }
}

function findLinkFromClickTarget(target) {
  return findClosestRecursively(
    target,
    "a[href]:not([target^=_]):not([download])"
  );
}

function getLocationForLink(link) {
  return expandURL(link.getAttribute("href") || "");
}

function expandURL(locatable) {
  return new URL(locatable.toString(), document.baseURI);
}

function doesNotTargetIFrame(name) {
  if (name === "_blank") {
    return false;
  } else if (name) {
    for (const element of document.getElementsByName(name)) {
      if (element instanceof HTMLIFrameElement) return false;
    }

    return true;
  } else {
    return true;
  }
}
class LinkClickObserver {
  started = false;

  constructor(delegate, eventTarget) {
    this.delegate = delegate;
    this.eventTarget = eventTarget;
  }

  start() {
    if (!this.started) {
      this.eventTarget.addEventListener("click", this.clickCaptured, true);
      this.started = true;
    }
  }

  stop() {
    if (this.started) {
      this.eventTarget.removeEventListener("click", this.clickCaptured, true);
      this.started = false;
    }
  }

  clickCaptured = () => {
    this.eventTarget.removeEventListener("click", this.clickBubbled, false);
    this.eventTarget.addEventListener("click", this.clickBubbled, false);
  };

  clickBubbled = (event) => {
    if (event instanceof MouseEvent && this.clickEventIsSignificant(event)) {
      const target =
        (event.composedPath && event.composedPath()[0]) || event.target;
      const link = findLinkFromClickTarget(target);
      if (link && doesNotTargetIFrame(link.target)) {
        const location = getLocationForLink(link);
        if (this.delegate.willFollowLinkToLocation(link, location, event)) {
          let command = undefined;
          let params = {};
          let id = undefined;
          [command, id] = splitPath(location);
          params = {
            ...Object.fromEntries(
              Array.from(location.searchParams.entries()).map(
                ([key, value]) => {
                  const parsed = parseInt(value);
                  return [key, isNaN(parsed) ? value : parsed];
                }
              )
            ),
            id: parseInt(id),
          };
          turboInvoke(command, params);
        } else if (event.target.getAttribute("command") == "open_url") {
          let command = "open_url";
          let params = { url: event.target.href };
          turboInvoke(command, params);
        }
      }
    }
  };

  clickEventIsSignificant(event) {
    return !(
      (event.target && event.target.isContentEditable) ||
      event.which > 1 ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey ||
      event.shiftKey
    );
  }
}

LinkClickObserver = new LinkClickObserver(Turbo.session, window);

LinkClickObserver.start();

document.addEventListener("DOMContentLoaded", (event) => {
  turboInvoke("index");
});

window.addEventListener("click", function (event) {
  // if (event.target.tagName !== "BUTTON") {
  //   console.log("preventDefault", event.target.tagName);
  //   event.preventDefault();
  // }
  const link = event.target.closest("a");
  if (link) {
    event.preventDefault();
  }
  // Rework this to make it be powered by turbo.js
  // if (event.target.tagName === "A" && event.target.href != undefined) {
  //   turboInvoke("open_browser", { url: event.target.href });
  // }
});

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

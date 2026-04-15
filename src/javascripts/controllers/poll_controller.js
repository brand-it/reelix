import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="poll"
//
// Periodically invokes a Tauri command and processes the response via Turbo.
// An empty response means "still pending" — keep polling.
// Any non-empty Turbo stream response navigates the app forward.
//
// Usage:
// <div data-controller="poll"
//      data-poll-command-value="poll_auth_token"
//      data-poll-interval-value="5000">
// </div>

export default class extends Controller {
  static values = {
    command: String,
    interval: { type: Number, default: 5000 },
  };

  connect() {
    this.poll();
  }

  disconnect() {
    clearTimeout(this.timeout);
  }

  async poll() {
    try {
      const response = await window.__TAURI__.core.invoke(this.commandValue);
      if (response && response.trim().length > 0) {
        window.processTurboResponse(response);
        return;
      }
    } catch (e) {
      console.error(`poll controller: command=${this.commandValue} error=${e}`);
    }

    this.timeout = setTimeout(() => this.poll(), this.intervalValue);
  }
}

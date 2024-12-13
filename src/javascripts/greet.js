import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="greet"
export default class extends Controller {
  static targets = ["input", "message"];

  async greet(event) {
    // Prevent the default form submission behavior
    event.preventDefault();

    const name = this.inputTarget.value;

    // Call the Tauri Rust command and update the message target
    const message = await window.__TAURI__.core.invoke("greet", { name });
    // message is html
    this.messageTarget.innerHTML = message;
  }
}

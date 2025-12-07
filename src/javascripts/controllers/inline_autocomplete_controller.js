import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="disk-select"
export default class extends Controller {
  static targets = ["input", "placeholder"];

  connect() {
    this.inputTargets.forEach((input) => {
      input.addEventListener("keyup", (event) => this.submit(input, event));
      input.addEventListener("keydown", (event) =>
        this.autocomplete(input, event)
      );
    });
  }

  submit(input, event) {
    if (event.key !== "Tab") {
      const value = input.value;
      window.turboInvoke("suggestion", { search: value });
    }
  }

  autocomplete(input, event) {
    if (event.key === "Tab") {
      const placeholder = this.placeholderTarget?.textContent?.trim();
      if (placeholder) {
        event.preventDefault();
        input.value = placeholder;
      }
    }
  }
}

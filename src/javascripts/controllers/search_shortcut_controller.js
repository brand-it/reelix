import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["input"];

  connect() {
    this.boundHandleKeydown = this.handleKeydown.bind(this);
    document.addEventListener("keydown", this.boundHandleKeydown);
  }

  disconnect() {
    document.removeEventListener("keydown", this.boundHandleKeydown);
  }

  handleKeydown(event) {
    if ((event.metaKey || event.ctrlKey) && event.key === "f") {
      event.preventDefault();
      this.focusSearch();
    }
  }

  focusSearch() {
    if (this.hasInputTarget) {
      this.inputTarget.focus();
      this.inputTarget.select();
    }
  }
}

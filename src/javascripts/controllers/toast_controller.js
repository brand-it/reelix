import { Controller } from "@hotwired/stimulus";

// Manages Bootstrap toast lifecycle
// Connects to data-controller="toast"
export default class extends Controller {
  connect() {
    // Initialize the Bootstrap toast
    const toast = new bootstrap.Toast(this.element);

    // Show the toast
    toast.show();

    // Remove the element from DOM after it's hidden
    this.element.addEventListener("hidden.bs.toast", () => {
      this.element.remove();
    });
  }
}

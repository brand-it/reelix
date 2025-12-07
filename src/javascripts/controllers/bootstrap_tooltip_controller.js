import { Controller } from "@hotwired/stimulus";

// This will only hook up tool tips on pages that have this controller included
// Connects to data-controller="bootstrap-tooltip"
export default class extends Controller {
  connect() {
    const tooltipTriggerList = document.querySelectorAll(
      '[data-bs-toggle="tooltip"]'
    );
    [...tooltipTriggerList].map(
      (tooltipTriggerEl) => new bootstrap.Tooltip(tooltipTriggerEl)
    );
  }
}

import { Controller } from "@hotwired/stimulus"
import debounce from "lodash/debounce";

// Connects to data-controller="submit-on-keyup"
export default class extends Controller {
  static targets = ["input"]

  connect() {
    this.lastSubmittedValue = this.inputTarget.value;
    this.submitWithDebounce = debounce(this.submitWithDebounce.bind(this), 300);
  }

  submitWithDebounce(event) {
    event.preventDefault();

    if (this.inputTarget.value !== this.lastSubmittedValue) {
      this.lastSubmittedValue = this.inputTarget.value;
      this.element.requestSubmit();
    }
  }
}

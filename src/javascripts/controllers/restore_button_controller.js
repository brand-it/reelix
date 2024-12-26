import { Controller } from "@hotwired/stimulus"


// Connects to data-controller="restore-button"
export default class extends Controller {
  // static targets = ["input"]

  // connect() {
  //   this.lastSubmittedValue = this.inputTarget.value;
  //   this.submitWithDebounce = debounce(this.submitWithDebounce.bind(this), 300);
  // }

  click(event) {
    event.preventDefault();
    history.back();
  }
}

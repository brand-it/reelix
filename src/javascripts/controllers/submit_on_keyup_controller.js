import { Controller } from "@hotwired/stimulus";
import debounce from "lodash/debounce";

// Connects to data-controller="submit-on-keyup"
//
// Usage:
// <div data-controller="submit-on-keyup">
//   <form data-submit-on-keyup-target="form">
//     <input type="text" name="field1" data-submit-on-keyup-target="input">
//     <input type="text" name="field2" data-submit-on-keyup-target="input">
//     ...
//   </form>
// </div>
//
// Description:
// This controller listens for keyup events on multiple input fields.
// If any input field's value changes (compared to its last submitted state),
// the form will be automatically submitted after a 300ms debounce delay.
//
// Requirements:
// - Each input should have a unique `name` or `id` attribute.
// - Debouncing is handled using lodash.debounce (install with `npm install lodash.debounce`)

export default class extends Controller {
  static targets = ["input", "form"];

  connect() {
    this.lastSubmittedValues = new Map();
    this.submitWithDebounce = debounce(
      this.submitWithDebounceActual.bind(this),
      300
    );
    this.inputTargets.forEach((input) => {
      this.lastSubmittedValues.set(input.name || input.id, input.value);
      input.addEventListener(
        "keyup",
        this.submitWithDebounce.bind(this, input)
      );
    });
  }

  submitWithDebounceActual(input, event) {
    const key = input.name || input.id;
    const currentValue = input.value;

    if (this.lastSubmittedValues.get(key) !== currentValue) {
      event.preventDefault();
      this.lastSubmittedValues.set(key, currentValue);
      this.formTarget.requestSubmit();
    }
  }
}

import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="disk-select"
export default class extends Controller {
  static targets = ["selectedDropdown", "item"];
  static selectedClass = "dropdown-selected";

  // data-action="click->disk-select#selected"
  selected(event) {
    const selectedItem = event.currentTarget;
    this.itemTargets.forEach((item) => {
      item.classList.remove(this.selectedClass);
    });
    selectedItem.classList.add(this.selectedClass);
    this.selectedDropdownTarget.innerHTML = selectedItem.innerHTML;
    window.turboInvoke("selected_disk", {
      diskId: parseInt(selectedItem.dataset.value),
    });
  }
}

import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="rip-one"
export default class extends Controller {
  static targets = ["movieId", "link"];

  rip(event) {
    event.preventDefault();
    const button = event.currentTarget;

    const commandArgs = {
      diskId: parseInt(button.dataset.diskId),
      titleId: parseInt(button.dataset.titleId),
      mvdbId: parseInt(this.movieIdTarget.value),
    };

    turboInvoke("rip_one", commandArgs)
      .then((response) => {
        // console.log("Rip command sent successfully:", response);
      })
      .catch((error) => {
        console.error("Rip command failed:", error);
      });
  }
}

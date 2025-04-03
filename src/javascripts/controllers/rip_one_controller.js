import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="rip-one"
export default class extends Controller {
  static targets = ["movieId", "link"];

  rip(event) {
    event.preventDefault();
    const button = event.currentTarget;

    const commandArgs = {
      diskId: button.dataset.diskId,
      titleId: button.dataset.titleId,
      mvdbId: this.movieIdTarget.value,
    };
    console.log(commandArgs);

    turboInvoke("rip_one", commandArgs)
      .then((response) => {
        console.log("Rip command sent successfully:", response);
      })
      .catch((error) => {
        console.error("Rip command failed:", error);
      });
  }
}

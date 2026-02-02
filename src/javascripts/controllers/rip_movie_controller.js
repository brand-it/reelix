import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="rip-movie"
export default class extends Controller {
  static targets = ["movieId", "link", "part", "edition"];

  rip(event) {
    event.preventDefault();
    const button = event.currentTarget;

    // Find the card containing this button
    const card = button.closest(".movie-card");
    const partInput = card.querySelector('[data-rip-movie-target="part"]');
    const editionInput = card.querySelector(
      '[data-rip-movie-target="edition"]',
    );

    const commandArgs = {
      diskId: parseInt(button.dataset.diskId),
      titleId: parseInt(button.dataset.titleId),
      mvdbId: parseInt(this.movieIdTarget.value),
      part: partInput && partInput.value ? parseInt(partInput.value) : null,
      edition:
        editionInput && editionInput.value.trim()
          ? editionInput.value.trim()
          : null,
    };

    turboInvoke("rip_movie", commandArgs);
  }
}

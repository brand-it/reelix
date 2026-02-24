import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="episode-reorder"
export default class extends Controller {
  static targets = ["mapping"];
  static values = {
    mvdbId: Number,
    seasonNumber: Number,
  };

  // data-action="click->episode-reorder#apply"
  apply() {
    const swaps = this.mappingTargets
      .map((select) => {
        const from = parseInt(select.dataset.episodeNumber);
        const to = parseInt(select.value);
        if (Number.isNaN(from) || Number.isNaN(to) || from === to) {
          return null;
        }
        return { from, to };
      })
      .filter((swap) => swap !== null);

    window.turboInvoke("reorder_tv_episodes_on_ftp", {
      mvdbId: this.mvdbIdValue,
      seasonNumber: this.seasonNumberValue,
      swaps,
    });
  }

  // data-action="click->episode-reorder#reset"
  reset() {
    this.mappingTargets.forEach((select) => {
      select.value = "";
    });
  }
}

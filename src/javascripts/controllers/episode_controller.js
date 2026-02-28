import { Controller } from "@hotwired/stimulus";

// Connects to data-controller="episode"
export default class extends Controller {
  // data-episode-target="selector"
  static targets = ["selector"];
  static values = {
    episodeNumber: Number,
    seasonNumber: Number,
    mvdbId: Number,
  };

  // data-action="click->episode#titleSelected"
  titleSelected(event) {
    const selectedTitle = event.currentTarget;
    const part = parseInt(selectedTitle.dataset.episodePart);
    const titleId = parseInt(selectedTitle.value);
    window.turboInvoke("assign_episode_to_title", {
      mvdbId: this.mvdbIdValue,
      seasonNumber: this.seasonNumberValue,
      episodeNumber: this.episodeNumberValue,
      titleId: titleId,
      part: part,
    });
  }
}

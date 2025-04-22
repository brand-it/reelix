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
    const previousValue = parseInt(selectedTitle.dataset.episodePreviousValue);
    const currentValue = parseInt(selectedTitle.value);
    if (!Number.isNaN(previousValue)) {
      window.turboInvoke("withdraw_episode_from_title", {
        mvdbId: this.mvdbIdValue,
        seasonNumber: this.seasonNumberValue,
        episodeNumber: this.episodeNumberValue,
        titleId: previousValue,
      });
      selectedTitle.dataset.episodePreviousValue = "";
    }
    if (!Number.isNaN(currentValue)) {
      window.turboInvoke("assign_episode_to_title", {
        mvdbId: this.mvdbIdValue,
        seasonNumber: this.seasonNumberValue,
        episodeNumber: this.episodeNumberValue,
        titleId: currentValue,
        part: part,
      });
      selectedTitle.dataset.episodePreviousValue = currentValue;
    }
  }
  // data-action="click->episode#rip"
  rip(event) {
    event.preventDefault();
    window.turboInvoke("rip_season");
  }
}

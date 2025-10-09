import { application } from "./application.js";

import SubmitOnKeyupController from "./submit_on_keyup_controller.js";
application.register("submit-on-keyup", SubmitOnKeyupController);

import RestoreButtonController from "./restore_button_controller.js";
application.register("restore-button", RestoreButtonController);

import RipOneController from "./rip_movie_controller.js";
application.register("rip-movie", RipOneController);

import DiskSelectController from "./disk_select_controller.js";
application.register("disk-select", DiskSelectController);

import EpisodeController from "./episode_controller.js";
application.register("episode", EpisodeController);

import BoostrapTooltipController from "./boostrap_tooltip_controller.js";
application.register("boostrap-tooltip", BoostrapTooltipController);

import { application } from "./application.js";

import SubmitOnKeyupController from "./submit_on_keyup_controller.js";
application.register("submit-on-keyup", SubmitOnKeyupController);

import RestoreButtonController from "./restore_button_controller.js";
application.register("restore-button", RestoreButtonController);

import RipOneController from "./rip_one_controller.js";
application.register("rip-one", RipOneController);

import DiskSelectController from "./disk_select_controller.js";
application.register("disk-select", DiskSelectController);

import EpisodeController from "./episode_controller.js";
application.register("episode", EpisodeController);

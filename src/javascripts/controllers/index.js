import { application } from "./application.js";

import SubmitOnKeyupController from "./submit_on_keyup_controller.js";
application.register("submit-on-keyup", SubmitOnKeyupController);

import RestoreButtonController from "./restore_button_controller.js";
application.register("restore-button", RestoreButtonController);

import RipOneController from "./rip_one_controller.js";
application.register("rip-one", RipOneController);

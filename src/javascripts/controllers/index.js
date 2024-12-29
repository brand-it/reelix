import { application } from "./application.js";

import SubmitOnKeyupController from "./submit_on_keyup_controller.js";
application.register("submit-on-keyup", SubmitOnKeyupController);

import RestoreButtonController from "./restore_button_controller.js";
application.register("restore-button", RestoreButtonController);

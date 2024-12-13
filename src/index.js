// Style Sheets
import "./stylesheets/application.scss";

// Images
import "./images/javascript.svg"
import "./images/tauri.svg"

// Javascripts
import { application } from "./javascripts/stimulus.js"
import './javascripts/greet.js'
import './javascripts/turbo.js'

import Greet from './javascripts/greet.js'
application.register("greet", Greet)

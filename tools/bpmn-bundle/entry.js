// Einstieg für das Bundle unter public/vendor/bpmn/ (Bau: siehe README.md in diesem Ordner).
// Stellt den Modellierer und das Modul für die Token-Simulation global bereit;
// verwendet werden sie in public/bpmn-editor.js.
import BpmnModeler from 'bpmn-js/lib/Modeler';
import TokenSimulationModule from 'bpmn-js-token-simulation';

import 'bpmn-js/dist/assets/diagram-js.css';
import 'bpmn-js/dist/assets/bpmn-js.css';
import 'bpmn-js/dist/assets/bpmn-font/css/bpmn.css';
import 'bpmn-js-token-simulation/assets/css/bpmn-js-token-simulation.css';

window.BpmnJS = BpmnModeler;
window.BpmnTokenSimulation = TokenSimulationModule;

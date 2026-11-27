// Brücke zwischen der Leptos-Oberfläche und dem BPMN-2.0-Editor bpmn-js (https://bpmn.io).
//
// Die Bibliothek (bpmn-js mit Token-Simulation) liegt als eigenes Bundle unter /vendor/bpmn/
// (gebaut in tools/bpmn-bundle) und wird erst geladen, wenn ein Editor geöffnet wird.
// Rust ruft die Funktionen unter `window.hbeBpmn` auf (siehe src/components/bpmn/components/editor_js.rs).
(function () {
  "use strict";

  var BASE = "/vendor/bpmn/";

  // Deutsche Texte für Palette, Kontextmenü und Elementauswahl.
  var GERMAN = {
    "Activate hand tool": "Hand-Werkzeug (verschieben)",
    "Activate lasso tool": "Lasso-Werkzeug (auswählen)",
    "Activate create/remove space tool": "Platz schaffen oder entfernen",
    "Activate global connect tool": "Verbindungs-Werkzeug",
    "Create start event": "Startereignis anlegen",
    "Create intermediate/boundary event": "Zwischenereignis anlegen",
    "Create end event": "Endereignis anlegen",
    "Create gateway": "Gateway anlegen",
    "Create task": "Aufgabe anlegen",
    "Create data object reference": "Datenobjekt anlegen",
    "Create data store reference": "Datenspeicher anlegen",
    "Create expanded sub-process": "Aufgeklappten Teilprozess anlegen",
    "Create pool/participant": "Pool/Teilnehmer anlegen",
    "Create group": "Gruppe anlegen",
    "Append end event": "Endereignis anhängen",
    "Append gateway": "Gateway anhängen",
    "Append task": "Aufgabe anhängen",
    "Append intermediate/boundary event": "Zwischenereignis anhängen",
    "Append receive task": "Empfangsaufgabe anhängen",
    "Append message intermediate catch event": "Nachrichten-Zwischenereignis anhängen",
    "Append timer intermediate catch event": "Zeitgeber-Zwischenereignis anhängen",
    "Append conditional intermediate catch event": "Bedingungs-Zwischenereignis anhängen",
    "Append signal intermediate catch event": "Signal-Zwischenereignis anhängen",
    "Append compensation activity": "Kompensationsaktivität anhängen",
    "Add text annotation": "Textanmerkung hinzufügen",
    "Add lane above": "Bahn oberhalb hinzufügen",
    "Add lane below": "Bahn unterhalb hinzufügen",
    "Divide into two lanes": "In zwei Bahnen teilen",
    "Divide into three lanes": "In drei Bahnen teilen",
    "Change element": "Element ändern",
    "Change type": "Typ ändern",
    "Connect to other element": "Mit anderem Element verbinden",
    "Connect using association": "Mit Assoziation verbinden",
    "Connect using data input association": "Mit Dateneingabe verbinden",
    "Delete": "Löschen",
    "Align elements": "Elemente ausrichten",
    "Distribute elements horizontally": "Horizontal verteilen",
    "Distribute elements vertically": "Vertikal verteilen",
    "Search in diagram": "Im Diagramm suchen",
    "Open {element}": "{element} öffnen",
    "Toggle non-interrupting": "Nicht unterbrechend umschalten",
    "Loop": "Schleife",
    "Parallel multi-instance": "Mehrfach parallel",
    "Sequential multi-instance": "Mehrfach sequenziell",
    "Collection": "Sammlung",
    "Participant multiplicity": "Mehrere Teilnehmer",
    "Task": "Aufgabe",
    "User task": "Benutzeraufgabe",
    "Manual task": "Manuelle Aufgabe",
    "Service task": "Serviceaufgabe",
    "Script task": "Skriptaufgabe",
    "Business rule task": "Geschäftsregelaufgabe",
    "Send task": "Sendeaufgabe",
    "Receive task": "Empfangsaufgabe",
    "Call activity": "Aufruf-Aktivität",
    "Sub-process (collapsed)": "Teilprozess (zugeklappt)",
    "Sub-process (expanded)": "Teilprozess (aufgeklappt)",
    "Event sub-process": "Ereignis-Teilprozess",
    "Transaction": "Transaktion",
    "Ad-hoc sub-process": "Ad-hoc-Teilprozess",
    "Exclusive gateway": "Exklusives Gateway",
    "Parallel gateway": "Paralleles Gateway",
    "Inclusive gateway": "Inklusives Gateway",
    "Complex gateway": "Komplexes Gateway",
    "Event-based gateway": "Ereignisbasiertes Gateway",
    "Start event": "Startereignis",
    "End event": "Endereignis",
    "Intermediate throw event": "Auslösendes Zwischenereignis",
    "Message start event": "Nachrichten-Startereignis",
    "Timer start event": "Zeitgeber-Startereignis",
    "Conditional start event": "Bedingungs-Startereignis",
    "Signal start event": "Signal-Startereignis",
    "Message end event": "Nachrichten-Endereignis",
    "Escalation end event": "Eskalations-Endereignis",
    "Error end event": "Fehler-Endereignis",
    "Compensation end event": "Kompensations-Endereignis",
    "Signal end event": "Signal-Endereignis",
    "Terminate end event": "Terminierungs-Endereignis",
    "Message intermediate catch event": "Nachrichten-Zwischenereignis (eintretend)",
    "Message intermediate throw event": "Nachrichten-Zwischenereignis (auslösend)",
    "Timer intermediate catch event": "Zeitgeber-Zwischenereignis",
    "Escalation intermediate throw event": "Eskalations-Zwischenereignis",
    "Conditional intermediate catch event": "Bedingungs-Zwischenereignis",
    "Link intermediate catch event": "Link-Zwischenereignis (eintretend)",
    "Link intermediate throw event": "Link-Zwischenereignis (auslösend)",
    "Compensation intermediate throw event": "Kompensations-Zwischenereignis",
    "Signal intermediate catch event": "Signal-Zwischenereignis (eintretend)",
    "Signal intermediate throw event": "Signal-Zwischenereignis (auslösend)",
    "Sequence flow": "Sequenzfluss",
    "Default flow": "Standardfluss",
    "Conditional flow": "Bedingter Fluss",
    "Data object reference": "Datenobjekt",
    "Data store reference": "Datenspeicher",
    "Expanded pool/participant": "Aufgeklappter Pool",
    "Empty pool/participant": "Leerer Pool",
    "Tasks": "Aufgaben",
    "Gateways": "Gateways",
    "Events": "Ereignisse",
    "Sub-processes": "Teilprozesse",
    "Data objects": "Datenobjekte",
    "Participants": "Teilnehmer",
    // Token-Simulation
    "Token Simulation": "Token-Simulation",
    "Play Simulation": "Simulation fortsetzen",
    "Pause Simulation": "Simulation anhalten",
    "Play/Pause Simulation": "Simulation fortsetzen/anhalten",
    "Reset Simulation": "Simulation zurücksetzen",
    "Toggle Simulation Log": "Protokoll ein-/ausblenden",
    "Simulation Log": "Simulationsprotokoll",
    "Set animation speed = {speed}": "Geschwindigkeit {speed}",
    "Set Sequence Flow": "Ausgehenden Fluss wählen",
    "Trigger Event": "Ereignis auslösen",
    "Focus process instance {scopeId}": "Prozessinstanz {scopeId} hervorheben",
    "Found unsupported elements": "Nicht unterstützte Elemente gefunden",
    "Not supported": "Nicht unterstützt",
    "No Entries": "Keine Einträge",
    "Close": "Schliessen",
    "Finished": "Beendet",
    "{name} started": "{name} gestartet",
    "{name} finished": "{name} beendet",
    "{name} canceled": "{name} abgebrochen",
    "Process started": "Prozess gestartet",
    "Process finished": "Prozess beendet",
    "Process canceled": "Prozess abgebrochen",
    "SubProcess started": "Teilprozess gestartet",
    "SubProcess finished": "Teilprozess beendet",
    "SubProcess canceled": "Teilprozess abgebrochen",
    "Slow": "langsam",
    "Normal": "normal",
    "Fast": "schnell",
    "Start Event": "Startereignis",
    "End Event": "Endereignis",
    "Boundary Event": "Randereignis",
    "Intermediate Event": "Zwischenereignis",
    "Exclusive Gateway": "Exklusives Gateway",
    "Inclusive Gateway": "Inklusives Gateway",
    "Parallel Gateway": "Paralleles Gateway",
    "User Task": "Benutzeraufgabe",
    "Manual Task": "Manuelle Aufgabe",
    "Service Task": "Serviceaufgabe",
    "Script Task": "Skriptaufgabe",
    "Business Rule Task": "Geschäftsregelaufgabe",
    "Send Task": "Sendeaufgabe",
    "Receive Task": "Empfangsaufgabe",
    "Call Activity": "Aufruf-Aktivität",
  };

  function translate(template, replacements) {
    replacements = replacements || {};
    var text = GERMAN[template] || template;
    return text.replace(/{([^}]+)}/g, function (_, key) {
      return replacements[key] !== undefined ? replacements[key] : "{" + key + "}";
    });
  }

  var loading = null;

  function loadLibrary() {
    if (window.BpmnJS) return Promise.resolve();
    if (loading) return loading;
    var link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = BASE + "bpmn.css";
    document.head.appendChild(link);
    loading = new Promise(function (resolve, reject) {
      var script = document.createElement("script");
      script.src = BASE + "bpmn.js";
      script.onload = function () { resolve(); };
      script.onerror = function () {
        loading = null;
        script.remove();
        reject(new Error("Der BPMN-Editor konnte nicht geladen werden."));
      };
      document.head.appendChild(script);
    });
    return loading;
  }

  // Es ist immer höchstens ein Editor offen.
  var current = null;

  function reportError(onError, prefix, err) {
    var message = err && err.message ? err.message : String(err);
    onError(prefix + message);
  }

  function emitChange(editor) {
    editor.modeler.saveXML({ format: true }).then(function (result) {
      if (current === editor) editor.onChange(result.xml);
    });
  }

  function importXml(editor, xml, onDone) {
    return editor.modeler.importXML(xml).then(function (result) {
      if (current !== editor) return;
      editor.modeler.get("canvas").zoom("fit-viewport", "auto");
      if (result.warnings && result.warnings.length) {
        editor.onError("Diagramm mit " + result.warnings.length + " Warnung(en) geladen; nicht alle Elemente werden unterstützt.");
      }
      if (onDone) onDone();
    });
  }

  function beforeUnload(event) {
    if (current && current.dirty) {
      event.preventDefault();
      event.returnValue = "";
    }
  }

  function keydown(event) {
    if (!current) return;
    if ((event.ctrlKey || event.metaKey) && !event.shiftKey && !event.altKey && event.key.toLowerCase() === "s") {
      event.preventDefault();
      current.onSave();
    }
  }

  window.addEventListener("beforeunload", beforeUnload);
  window.addEventListener("keydown", keydown);

  function download(name, content, type) {
    var url = URL.createObjectURL(new Blob([content], { type: type }));
    var a = document.createElement("a");
    a.href = url;
    a.download = name;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
  }

  function safeFileName(name) {
    return (name || "diagramm").replace(/[\\/:*?"<>|]+/g, "_").trim() || "diagramm";
  }

  window.hbeBpmn = {
    // Erstellt den Editor im Element `container` und lädt das XML.
    // onChange(xml) bei jeder Änderung, onSave() bei Strg+S, onError(text) bei Problemen,
    // onSimulation(aktiv) beim Ein- und Ausschalten der Token-Simulation.
    mount: function (container, xml, onChange, onSave, onError, onSimulation) {
      this.destroy();
      var editor = {
        modeler: null, onChange: onChange, onSave: onSave, onError: onError, onSimulation: onSimulation,
        dirty: false, simulating: false,
      };
      current = editor;
      loadLibrary()
        .then(function () {
          if (current !== editor) return;
          editor.modeler = new window.BpmnJS({
            container: container,
            additionalModules: [window.BpmnTokenSimulation, { translate: ["value", translate] }],
          });
          editor.modeler.on("tokenSimulation.toggleMode", function (event) {
            editor.simulating = !!event.active;
            if (current === editor) editor.onSimulation(editor.simulating);
          });
          editor.modeler.on("commandStack.changed", function (event) {
            // Beim Laden wird der Verlauf geleert; das ist keine Änderung.
            if (event.trigger === "clear") return;
            editor.dirty = true;
            emitChange(editor);
          });
          return importXml(editor, xml);
        })
        .catch(function (err) {
          if (current === editor) reportError(onError, "", err);
        });
    },

    destroy: function () {
      if (current && current.modeler) current.modeler.destroy();
      current = null;
    },

    markSaved: function () {
      if (current) current.dirty = false;
    },

    undo: function () {
      if (current && current.modeler && !current.simulating) current.modeler.get("commandStack").undo();
    },

    redo: function () {
      if (current && current.modeler && !current.simulating) current.modeler.get("commandStack").redo();
    },

    // Schaltet die Token-Simulation ein oder aus. Im Simulationsmodus ist das Modell gesperrt;
    // ein Klick auf das Play-Symbol am Startereignis schickt einen Token durch den Prozess.
    toggleSimulation: function () {
      if (current && current.modeler) current.modeler.get("toggleMode").toggleMode();
    },

    zoom: function (step) {
      if (!current || !current.modeler) return;
      if (step === 0) current.modeler.get("canvas").zoom("fit-viewport", "auto");
      else current.modeler.get("zoomScroll").stepZoom(step);
    },

    // Öffnet die Dateiauswahl und ersetzt das Diagramm durch die gewählte .bpmn-Datei.
    importFile: function () {
      var editor = current;
      if (!editor || !editor.modeler || editor.simulating) return;
      var input = document.createElement("input");
      input.type = "file";
      input.accept = ".bpmn,.xml,application/xml,text/xml";
      input.onchange = function () {
        var file = input.files && input.files[0];
        if (!file) return;
        file.text().then(function (text) {
          return importXml(editor, text, function () {
            // Ein Import zählt als Änderung, die gespeichert werden muss.
            editor.dirty = true;
            editor.onChange(text);
          });
        }).catch(function (err) {
          reportError(editor.onError, "Import fehlgeschlagen: ", err);
        });
      };
      input.click();
    },

    exportXml: function (name) {
      var editor = current;
      if (!editor || !editor.modeler) return;
      editor.modeler.saveXML({ format: true })
        .then(function (result) { download(safeFileName(name) + ".bpmn", result.xml, "application/xml"); })
        .catch(function (err) { reportError(editor.onError, "Export fehlgeschlagen: ", err); });
    },

    exportSvg: function (name) {
      var editor = current;
      if (!editor || !editor.modeler) return;
      editor.modeler.saveSVG()
        .then(function (result) { download(safeFileName(name) + ".svg", result.svg, "image/svg+xml"); })
        .catch(function (err) { reportError(editor.onError, "Export fehlgeschlagen: ", err); });
    },
  };
})();

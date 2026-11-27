# BPMN-Bundle

Baut den grafischen Editor [bpmn-js](https://bpmn.io) zusammen mit der Erweiterung
[bpmn-js-token-simulation](https://github.com/bpmn-io/bpmn-js-token-simulation) zu einem Bundle.
Die Token-Simulation gibt es nur als ES-Module, deshalb wird nicht vom CDN geladen.

Das Ergebnis liegt eingecheckt unter `public/vendor/bpmn/` (`bpmn.js`, `bpmn.css`, Schriften,
`LICENSES.txt`). Für den normalen Build und das Docker-Image ist dieser Ordner nicht nötig;
neu bauen nur nach einer Versionsänderung in `package.json`:

```bash
cd tools/bpmn-bundle
npm install
npm run build
```

Danach `LICENSES.txt` aktualisieren, falls sich Lizenzen geändert haben.
Verwendet wird das Bundle in `public/bpmn-editor.js`.

use serde::{Deserialize, Serialize};

/// Eintrag in der Übersicht, ohne das XML.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BpmnDiagram {
    pub id: i64,
    pub name: String,
    pub description: String,
    /// Zeitpunkt der letzten Änderung, formatiert als "TT.MM.JJJJ HH:MM".
    pub updated: String,
}

/// Diagramm mit Inhalt für den Editor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BpmnDiagramContent {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub xml: String,
}

/// Leeres Diagramm mit einem Startereignis, damit der Editor nicht mit einer weissen Fläche beginnt.
pub fn empty_diagram(name: &str) -> String {
    let name = escape_xml(name);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI" xmlns:dc="http://www.omg.org/spec/DD/20100524/DC" id="Definitions_1" targetNamespace="http://bpmn.io/schema/bpmn" exporter="Workspace HBE">
  <bpmn:process id="Process_1" name="{name}" isExecutable="false">
    <bpmn:startEvent id="StartEvent_1" />
  </bpmn:process>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
      <bpmndi:BPMNShape id="StartEvent_1_di" bpmnElement="StartEvent_1">
        <dc:Bounds x="180" y="160" width="36" height="36" />
      </bpmndi:BPMNShape>
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>
"#
    )
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Grobe Prüfung, ob der Text ein BPMN-2.0-Dokument ist. Die genaue Prüfung macht der Editor beim Import.
pub fn looks_like_bpmn(xml: &str) -> bool {
    xml.contains("definitions") && xml.contains("http://www.omg.org/spec/BPMN/20100524/MODEL")
}

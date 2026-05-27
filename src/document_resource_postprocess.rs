//! Document resource postprocess/prewrite mirroring TS document-postprocess.ts and resource-postprocess.ts.
//!
//! Maps typed PSD document fields (e.g., variable_sets, data_sets, display_info,
//! custom_points) to/from the low-level image resource storage (XML strings, typed resources,
//! descriptor resources).

use crate::error::Result;
use crate::psd::Psd;

/// Apply read-side document resource postprocess: map low-level ImageResources fields
/// onto typed Psd fields.
pub fn apply_document_postprocess(psd: &mut Psd) -> Result<()> {
    if let Some(ref resources) = psd.image_resources {
        // Map clipping from resource 1026
        if let (Some(clipping), Some(layers)) = (resources.clipping.as_ref(), psd.children.as_mut())
        {
            for (layer, value) in layers.iter_mut().zip(clipping.iter()) {
                layer.clipping = Some(*value);
            }
        }

        // Map resource visibility from resource 1072
        if let (Some(visibility), Some(layers)) = (
            resources.resource_visibility_typed.as_ref(),
            psd.children.as_mut(),
        ) {
            for (layer, value) in layers.iter_mut().zip(visibility.values.iter()) {
                layer.resource_visible = Some(*value);
            }
        }

        // Map variables XML → typed variable_sets
        if let Some(ref xml) = resources.variables {
            psd.variable_sets = Some(parse_variables_xml(xml));
        }

        // Map data sets XML → typed data_sets
        if let Some(ref xml) = resources.data_sets {
            psd.data_sets = Some(parse_data_sets_xml(xml));
        }

        // Map descriptor resources 1065, 1074, 1075
        if let Some(desc) = resources.descriptor_resources.get(&1065) {
            psd.descriptor_1065 = Some(desc.clone());
        }
        if let Some(desc) = resources.descriptor_resources.get(&1074) {
            psd.descriptor_1074 = Some(desc.clone());
        }
        if let Some(desc) = resources.descriptor_resources.get(&1075) {
            psd.descriptor_1075 = Some(desc.clone());
        }

        if let Some(points) = resources.custom_points_typed.as_ref() {
            psd.custom_points = Some(points.points.clone());
        }

        if let Some(info) = resources.display_info_typed.as_ref() {
            psd.display_info = Some(crate::psd::DisplayInfo {
                h_res_unit: info.h_res_unit,
                v_res_unit: info.v_res_unit,
                width_unit: info.width_unit,
                height_unit: info.height_unit,
            });
        }
    }
    Ok(())
}

/// Apply write-side document resource prewrite: map typed Psd fields back
/// onto low-level ImageResources fields before serialization.
pub fn apply_document_prewrite(psd: &mut Psd) -> Result<()> {
    let resources = psd.image_resources.get_or_insert_with(Default::default);

    // Map clipping from layers → resource 1026
    if let Some(layers) = psd.children.as_ref() {
        let clipping_values: Vec<u16> = layers
            .iter()
            .map(|layer| layer.clipping.unwrap_or(0))
            .collect();
        if clipping_values.iter().any(|value| *value > 0) {
            resources.clipping = Some(clipping_values);
        }
    }

    // Map resource visibility from layers → resource 1072
    if let Some(layers) = psd.children.as_ref() {
        let values: Vec<bool> = layers
            .iter()
            .map(|layer| layer.resource_visible != Some(false))
            .collect();
        if values.iter().any(|value| !*value) {
            resources.resource_visibility_typed =
                Some(crate::image_resources::ResourceVisibility { values });
        }
    }

    // Map typed variable_sets → variables XML
    if let Some(variable_sets) = psd.variable_sets.as_ref() {
        resources.variables = Some(build_variables_xml(variable_sets));
    }

    // Map typed data_sets → data sets XML
    if let Some(data_sets) = psd.data_sets.as_ref() {
        resources.data_sets = Some(build_data_sets_xml(data_sets));
    }

    // Map descriptor resources 1065, 1074, 1075
    if let Some(ref desc) = psd.descriptor_1065 {
        resources.descriptor_resources.insert(1065, desc.clone());
    }
    if let Some(ref desc) = psd.descriptor_1074 {
        resources.descriptor_resources.insert(1074, desc.clone());
    }
    if let Some(ref desc) = psd.descriptor_1075 {
        resources.descriptor_resources.insert(1075, desc.clone());
    }

    if let Some(points) = psd.custom_points.as_ref() {
        resources.custom_points_typed = Some(crate::image_resources::CustomPointsResource {
            version: 3,
            points: points.clone(),
        });
    }

    if let Some(info) = psd.display_info.as_ref() {
        resources.display_info_typed = Some(crate::image_resources::DisplayInfoResource {
            version: 1,
            h_res_unit: info.h_res_unit,
            v_res_unit: info.v_res_unit,
            width_unit: info.width_unit,
            height_unit: info.height_unit,
        });
    }

    Ok(())
}

// ── XML helpers (porting TS document-postprocess.ts behavior) ────────────────

/// Parse variables XML into typed VariableSet vectors.
/// TS uses simple regex-based extraction from the XML string.
fn parse_variables_xml(xml: &str) -> Vec<crate::psd::VariableSet> {
    let mut variables = Vec::new();
    let mut pos = 0;
    let search = "<variable ";
    while pos < xml.len() {
        if let Some(start) = xml[pos..].find(search) {
            let abs_start = pos + start;
            if let Some(end) = xml[abs_start..].find("/>") {
                let tag = &xml[abs_start..abs_start + end];
                let var_name = extract_xml_attr(tag, "varName");
                let trait_name = extract_xml_attr(tag, "trait");
                let doc_ref = extract_xml_attr(tag, "docRef");
                let placement_method = extract_xml_attr(tag, "placementMethod");
                let align = extract_xml_attr(tag, "align");
                let valign = extract_xml_attr(tag, "valign");
                let clip = extract_xml_attr(tag, "clip");
                variables.push(crate::psd::VariableSet {
                    var_name,
                    trait_name,
                    doc_ref,
                    placement_method,
                    align,
                    valign,
                    clip,
                });
                pos = abs_start + end + 2;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    variables
}

/// Build variables XML from typed VariableSet vectors.
fn build_variables_xml(variables: &[crate::psd::VariableSet]) -> String {
    let mut xml = String::from("<variableSets><variableSet><variables>");
    for v in variables {
        xml.push_str("<variable");
        if let Some(ref name) = v.var_name {
            xml.push_str(&format!(" varName=\"{}\"", escape_xml(name)));
        }
        if let Some(ref trait_name) = v.trait_name {
            xml.push_str(&format!(" trait=\"{}\"", escape_xml(trait_name)));
        }
        if let Some(ref doc_ref) = v.doc_ref {
            xml.push_str(&format!(" docRef=\"{}\"", escape_xml(doc_ref)));
        }
        if let Some(ref pm) = v.placement_method {
            xml.push_str(&format!(" placementMethod=\"{}\"", escape_xml(pm)));
        }
        if let Some(ref a) = v.align {
            xml.push_str(&format!(" align=\"{}\"", escape_xml(a)));
        }
        if let Some(ref v) = v.valign {
            xml.push_str(&format!(" valign=\"{}\"", escape_xml(v)));
        }
        if let Some(ref c) = v.clip {
            xml.push_str(&format!(" clip=\"{}\"", escape_xml(c)));
        }
        xml.push_str("/>");
    }
    xml.push_str("</variables></variableSet></variableSets>");
    xml
}

/// Parse data sets XML into typed table: first row is header, subsequent rows are values.
fn parse_data_sets_xml(xml: &str) -> Vec<Vec<String>> {
    let mut table = vec![Vec::new()];
    let mut pos = 0;
    while let Some(rec_start) = xml[pos..].find("<sampleDataSet>") {
        let abs = pos + rec_start;
        if let Some(rec_end) = xml[abs..].find("</sampleDataSet>") {
            let record_xml = &xml[abs + "<sampleDataSet>".len()..abs + rec_end];
            let cells = extract_tagged_cells(record_xml);
            if !cells.is_empty() {
                if table[0].is_empty() {
                    table[0] = cells.iter().map(|(name, _)| name.clone()).collect();
                }
                table.push(cells.into_iter().map(|(_, value)| value).collect());
            }
            pos = abs + rec_end + "</sampleDataSet>".len();
        } else {
            break;
        }
    }
    table
}

/// Build data sets XML from typed table.
fn build_data_sets_xml(table: &[Vec<String>]) -> String {
    let mut xml = String::from("<sampleDataSets>");
    if let Some(header) = table.first() {
        for record in table.iter().skip(1) {
            xml.push_str("<sampleDataSet>");
            for (index, tag_name) in header.iter().enumerate() {
                let value = record.get(index).cloned().unwrap_or_default();
                xml.push_str(&format!(
                    "<{}>{}</{}>",
                    tag_name,
                    escape_xml(&value),
                    tag_name
                ));
            }
            xml.push_str("</sampleDataSet>");
        }
    }
    xml.push_str("</sampleDataSets>");
    xml
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn extract_xml_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = tag.find(&pattern) {
        let val_start = start + pattern.len();
        if let Some(end) = tag[val_start..].find('"') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    None
}

fn extract_tagged_cells(xml: &str) -> Vec<(String, String)> {
    let mut cols = Vec::new();
    let mut pos = 0;
    while let Some(cs) = xml[pos..].find('<') {
        let abs = pos + cs;
        if abs + 1 >= xml.len() || xml[abs + 1..].starts_with('/') {
            pos = abs + 1;
            continue;
        }
        if let Some(tag_end) = xml[abs + 1..].find('>') {
            let name = &xml[abs + 1..abs + 1 + tag_end];
            let close_tag = format!("</{}>", name);
            let content_start = abs + 1 + tag_end + 1;
            if let Some(close_offset) = xml[content_start..].find(&close_tag) {
                cols.push((
                    name.to_string(),
                    xml[content_start..content_start + close_offset].to_string(),
                ));
                pos = content_start + close_offset + close_tag.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    cols
}

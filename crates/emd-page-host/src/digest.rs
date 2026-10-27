// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Deterministic render evidence: every component of a page reduces to a
//! STABLE digest over its UI-neutral scene (the renderer owns geometry).
//!
//! Digest inputs are canonical: floats round-trip through fixed `{:.4}`
//! formatting, so a digest computed on two identical runs is bit-identical.
//! The FIXTURE STATE joins the page digest — an altered fixture state can
//! never look like an unaltered page.

use crate::error::PageError;
use emd_renderer::EmdComponent;
use emd_renderer::components::{build_plot_scene, sparkline_paths};
use serde_json::json;
use sha2::{Digest as _, Sha256};

fn fmt(value: f64) -> String {
    format!("{value:.4}")
}

/// Canonical string form of one component's rendered evidence.
pub fn canonical_render(component: &EmdComponent) -> Result<String, PageError> {
    let rendered = match component {
        EmdComponent::Plot(spec) => {
            let scene = build_plot_scene(spec).map_err(|message| PageError { message })?;
            let series: Vec<serde_json::Value> = scene
                .series
                .iter()
                .map(|s| {
                    json!({
                        "name": s.name, "color": s.color,
                        "line_paths": s.line_paths, "area_paths": s.area_paths,
                        "points": s.points.iter().map(|p| [fmt(p.x), fmt(p.y), p.tooltip.clone()]).collect::<Vec<_>>(),
                        "bars": s.bars.iter().map(|b| [fmt(b.x), fmt(b.y), fmt(b.width), fmt(b.height)]).collect::<Vec<_>>(),
                    })
                })
                .collect();
            json!({
                "scene": {
                    "width": fmt(scene.width), "height": fmt(scene.height),
                    "left": fmt(scene.left), "right": fmt(scene.right),
                    "top": fmt(scene.top), "bottom": fmt(scene.bottom),
                    "y_ticks": scene.y_ticks.iter().map(|t| [fmt(t.position), t.label.clone()]).collect::<Vec<_>>(),
                    "x_ticks": scene.x_ticks.iter().map(|t| [fmt(t.position), t.label.clone()]).collect::<Vec<_>>(),
                    "series": series,
                }
            })
            .to_string()
        }
        EmdComponent::Sparkline(spec) => {
            let paths = sparkline_paths(&spec.values, 300.0, 72.0);
            json!({
                "sparkline": {
                    "paths": paths,
                    "label": spec.label,
                    "value": spec.value,
                    "delta": spec.delta,
                    "null_slots": spec.values.iter().filter(|v| v.is_none()).count(),
                }
            })
            .to_string()
        }
        EmdComponent::Metric(spec) => json!({
            "metric": {
                "label": spec.label,
                "value": spec.value,
                "detail": spec.detail,
                "delta": spec.delta,
                "tone": format!("{:?}", spec.tone).to_lowercase(),
                "evidence_state": format!("{:?}", spec.evidence.state).to_lowercase(),
            }
        })
        .to_string(),
        EmdComponent::Query(spec) => json!({
            "query": {
                "title": spec.title, "language": spec.language,
                "source_char_len": spec.source.len(),
                "status": spec.status,
                "controls": spec.controls.iter().map(|c| json!([c.label.clone(), c.value.clone()])).collect::<Vec<_>>(),
            }
        })
        .to_string(),
        EmdComponent::DataGrid(spec) => json!({
            "datagrid": {
                "title": spec.title,
                "columns": spec.columns,
                "row_widths": spec.rows.iter().map(|r| r.len()).collect::<Vec<_>>(),
                "row_count": spec.rows.len(),
                "first_row": spec.rows.first(),
            }
        })
        .to_string(),
        EmdComponent::AgentFinding(spec) => json!({
            "agentfinding": {
                "title": spec.title, "summary": spec.summary,
                "finding_count": spec.findings.len(),
                "next_question": spec.next_question,
                "status": spec.status,
            }
        })
        .to_string(),
        EmdComponent::Grid(spec) => {
            let children: Result<Vec<String>, PageError> = spec
                .children
                .iter()
                .map(|child| component_digest(child))
                .collect();
            json!({
                "grid": {
                    "columns": spec.columns,
                    "gap_px": spec.gap_px,
                    "children": children?,
                }
            })
            .to_string()
        }
        EmdComponent::Panel(spec) => {
            let children: Result<Vec<String>, PageError> = spec
                .children
                .iter()
                .map(|child| component_digest(child))
                .collect();
            json!({
                "panel": {
                    "title": spec.title,
                    "subtitle": spec.subtitle,
                    "controls": spec.controls.iter().map(|c| json!([c.label.clone(), c.value.clone(), format!("{:?}", c.primary)])).collect::<Vec<_>>(),
                    "children": children?,
                }
            })
            .to_string()
        }
    };
    Ok(rendered)
}

/// Digest of one component (kind-prefixed canonical render).
pub fn component_digest(component: &EmdComponent) -> Result<String, PageError> {
    let kind = match component {
        EmdComponent::Grid(_) => "grid:",
        EmdComponent::Panel(_) => "panel:",
        EmdComponent::Plot(_) => "plot:",
        EmdComponent::Sparkline(_) => "sparkline:",
        EmdComponent::Metric(_) => "metric:",
        EmdComponent::Query(_) => "query:",
        EmdComponent::DataGrid(_) => "datagrid:",
        EmdComponent::AgentFinding(_) => "agentfinding:",
    };
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(canonical_render(component)?.as_bytes());
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

//! Typed extended-markdown components and deterministic visualization layout.
//!
//! EMD components are fenced JSON (` ```emd `). JSON is intentionally the
//! interchange format: an agent can author it, a person can inspect it, and a
//! renderer never has to execute notebook source. Live notebooks update the
//! JSON through their ordinary document-version refresh; parsing and layout
//! remain pure and bounded.

use serde::{Deserialize, Serialize};

pub const COMPONENT_VERSION: u16 = 1;
pub const MAX_COMPONENTS: usize = 64;
pub const MAX_SERIES: usize = 16;
pub const MAX_POINTS: usize = 2_048;
pub const MAX_TABLE_ROWS: usize = 500;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDocument {
    #[serde(default = "component_version")]
    pub version: u16,
    #[serde(flatten)]
    pub component: EmdComponent,
}

fn component_version() -> u16 {
    COMPONENT_VERSION
}

/// The analytical vocabulary shared by every EMD host.
///
/// `Grid` and `Panel` are composition, not domain widgets. The same tree can
/// therefore describe a tracing workbench, a finance review, or the Axiom-like
/// query/results/agent arrangement without teaching Markdown about any one app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "spec", rename_all = "kebab-case")]
pub enum EmdComponent {
    Grid(GridSpec),
    Panel(PanelSpec),
    Plot(PlotSpec),
    Sparkline(SparklineSpec),
    Metric(MetricSpec),
    Query(QuerySpec),
    DataGrid(DataGridSpec),
    AgentFinding(AgentFindingSpec),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridSpec {
    #[serde(default = "default_columns")]
    pub columns: u8,
    #[serde(default = "default_gap")]
    pub gap_px: u16,
    pub children: Vec<EmdComponent>,
}

fn default_columns() -> u8 {
    2
}

fn default_gap() -> u16 {
    12
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelSpec {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub controls: Vec<ControlSpec>,
    #[serde(default)]
    pub children: Vec<EmdComponent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlSpec {
    pub label: String,
    /// Opaque action name. A document-only host shows the control disabled;
    /// an app host may route it through its normal pane-action channel.
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSpec {
    /// The operational question this component exists to answer.
    pub question: String,
    /// The probe, file, stream, or query that produced the reading.
    pub source: String,
    /// The aggregation or observation window, such as `last 15 min`.
    pub window: String,
    /// A human-readable age or refresh contract, such as `4 s`.
    pub freshness: String,
    /// Unit of every numeric value unless a series overrides it.
    pub units: String,
    pub state: EvidenceState,
    /// A command or declarative query that reproduces the result.
    pub reproduction: String,
    #[serde(default)]
    pub observed_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceState {
    Observed,
    Collecting,
    Silent,
    Unavailable,
    Stale,
    Uninstrumented,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSpec {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub mark: PlotMark,
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
    #[serde(default)]
    pub include_zero: bool,
    #[serde(default = "default_plot_height")]
    pub height: u16,
    #[serde(default = "default_true")]
    pub legend: bool,
    pub series: Vec<PlotSeries>,
    pub evidence: EvidenceSpec,
}

fn default_plot_height() -> u16 {
    300
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlotMark {
    #[default]
    Line,
    Area,
    Bar,
    Point,
    Step,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSeries {
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub units: Option<String>,
    pub values: Vec<PlotPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPoint {
    pub x: String,
    /// `None` is a missing observation and draws a gap. It never means zero.
    pub y: Option<f64>,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SparklineSpec {
    pub label: String,
    pub values: Vec<Option<f64>>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub delta: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    pub evidence: EvidenceSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSpec {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub delta: Option<String>,
    #[serde(default)]
    pub tone: MetricTone,
    pub evidence: EvidenceSpec,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetricTone {
    #[default]
    Neutral,
    Good,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuerySpec {
    pub title: String,
    pub language: String,
    pub source: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub controls: Vec<ControlSpec>,
    pub evidence: EvidenceSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataGridSpec {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    #[serde(default)]
    pub compact: bool,
    pub evidence: EvidenceSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentFindingSpec {
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub next_question: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    pub evidence: EvidenceSpec,
}

pub fn parse_component(source: &str) -> Result<ComponentDocument, String> {
    if source.len() > 256 * 1024 {
        return Err("EMD component exceeds the 256 KiB source limit".to_string());
    }
    let document: ComponentDocument = serde_json::from_str(source)
        .map_err(|error| format!("invalid EMD component JSON: {error}"))?;
    if document.version != COMPONENT_VERSION {
        return Err(format!(
            "unsupported EMD component version {}; this renderer supports {}",
            document.version, COMPONENT_VERSION
        ));
    }
    let mut count = 0;
    validate_component(&document.component, &mut count)?;
    Ok(document)
}

fn validate_component(component: &EmdComponent, count: &mut usize) -> Result<(), String> {
    *count += 1;
    if *count > MAX_COMPONENTS {
        return Err(format!("EMD tree exceeds {MAX_COMPONENTS} components"));
    }
    match component {
        EmdComponent::Grid(spec) => {
            if !(1..=4).contains(&spec.columns) {
                return Err("EMD grid columns must be between 1 and 4".to_string());
            }
            for child in &spec.children {
                validate_component(child, count)?;
            }
        }
        EmdComponent::Panel(spec) => {
            for child in &spec.children {
                validate_component(child, count)?;
            }
        }
        EmdComponent::Plot(spec) => {
            validate_evidence(&spec.evidence)?;
            if spec.series.is_empty() {
                return Err("EMD plot needs at least one series".to_string());
            }
            if spec.series.len() > MAX_SERIES {
                return Err(format!("EMD plot exceeds {MAX_SERIES} series"));
            }
            if !(120..=720).contains(&spec.height) {
                return Err("EMD plot height must be between 120 and 720 px".to_string());
            }
            let points: usize = spec.series.iter().map(|series| series.values.len()).sum();
            if points > MAX_POINTS {
                return Err(format!("EMD plot exceeds {MAX_POINTS} points"));
            }
        }
        EmdComponent::Sparkline(spec) => {
            validate_evidence(&spec.evidence)?;
            if spec.values.len() > MAX_POINTS {
                return Err(format!("EMD sparkline exceeds {MAX_POINTS} points"));
            }
        }
        EmdComponent::Metric(spec) => validate_evidence(&spec.evidence)?,
        EmdComponent::Query(spec) => validate_evidence(&spec.evidence)?,
        EmdComponent::DataGrid(spec) => {
            validate_evidence(&spec.evidence)?;
            if spec.rows.len() > MAX_TABLE_ROWS {
                return Err(format!("EMD data grid exceeds {MAX_TABLE_ROWS} rows"));
            }
            if spec.rows.iter().any(|row| row.len() != spec.columns.len()) {
                return Err("every EMD data-grid row must match its column count".to_string());
            }
        }
        EmdComponent::AgentFinding(spec) => validate_evidence(&spec.evidence)?,
    }
    Ok(())
}

fn validate_evidence(evidence: &EvidenceSpec) -> Result<(), String> {
    for (name, value) in [
        ("question", evidence.question.as_str()),
        ("source", evidence.source.as_str()),
        ("window", evidence.window.as_str()),
        ("freshness", evidence.freshness.as_str()),
        ("units", evidence.units.as_str()),
        ("reproduction", evidence.reproduction.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("EMD evidence `{name}` must not be empty"));
        }
    }
    Ok(())
}

/// A UI-neutral SVG scene. The renderer owns scales, ticks, gaps, paths, and
/// the colourblind-safe default palette; a host only paints these primitives.
#[derive(Debug, Clone, PartialEq)]
pub struct PlotScene {
    pub width: f64,
    pub height: f64,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub y_ticks: Vec<SceneTick>,
    pub x_ticks: Vec<SceneTick>,
    pub series: Vec<SceneSeries>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneTick {
    pub position: f64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneSeries {
    pub name: String,
    pub color: String,
    pub line_paths: Vec<String>,
    pub area_paths: Vec<String>,
    pub points: Vec<ScenePoint>,
    pub bars: Vec<SceneBar>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenePoint {
    pub x: f64,
    pub y: f64,
    pub tooltip: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneBar {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub tooltip: String,
}

const PLOT_COLORS: [&str; 8] = [
    "#0072B2", "#E69F00", "#009E73", "#CC79A7", "#D55E00", "#56B4E9", "#F0E442", "#6C71C4",
];

pub fn build_plot_scene(spec: &PlotSpec) -> Result<PlotScene, String> {
    let observed: Vec<f64> = spec
        .series
        .iter()
        .flat_map(|series| series.values.iter().filter_map(|point| point.y))
        .filter(|value| value.is_finite())
        .collect();
    if observed.is_empty() {
        return Err("plot contains no observed numeric values".to_string());
    }

    let width = 900.0;
    let height = f64::from(spec.height);
    let left = 62.0;
    let right = width - 22.0;
    let top = 18.0;
    let bottom = height - 42.0;
    let mut y_min = observed.iter().copied().fold(f64::INFINITY, f64::min);
    let mut y_max = observed.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if spec.include_zero || matches!(spec.mark, PlotMark::Bar | PlotMark::Area) {
        y_min = y_min.min(0.0);
        y_max = y_max.max(0.0);
    }
    if (y_max - y_min).abs() < f64::EPSILON {
        let pad = y_max.abs().max(1.0) * 0.05;
        y_min -= pad;
        y_max += pad;
    }
    let raw_step = (y_max - y_min) / 4.0;
    let step = nice_step(raw_step);
    let domain_min = (y_min / step).floor() * step;
    let domain_max = (y_max / step).ceil() * step;
    let plot_width = right - left;
    let plot_height = bottom - top;
    let y_of =
        |value: f64| bottom - ((value - domain_min) / (domain_max - domain_min)) * plot_height;

    let mut y_ticks = Vec::new();
    let mut tick = domain_min;
    while tick <= domain_max + step * 0.25 && y_ticks.len() < 12 {
        y_ticks.push(SceneTick {
            position: y_of(tick),
            label: format_number(tick),
        });
        tick += step;
    }

    let max_points = spec
        .series
        .iter()
        .map(|series| series.values.len())
        .max()
        .unwrap_or(1)
        .max(1);
    let x_of = |index: usize| {
        if max_points <= 1 {
            left + plot_width / 2.0
        } else {
            left + (index as f64 / (max_points - 1) as f64) * plot_width
        }
    };
    let x_tick_count = max_points.min(6);
    let mut x_ticks = Vec::new();
    for slot in 0..x_tick_count {
        let index = if x_tick_count <= 1 {
            0
        } else {
            slot * (max_points - 1) / (x_tick_count - 1)
        };
        let label = spec
            .series
            .iter()
            .find_map(|series| series.values.get(index))
            .map(|point| point.x.clone())
            .unwrap_or_else(|| index.to_string());
        if !x_ticks.iter().any(|tick: &SceneTick| tick.label == label) {
            x_ticks.push(SceneTick {
                position: x_of(index),
                label,
            });
        }
    }

    let baseline = y_of(0.0_f64.clamp(domain_min, domain_max));
    let grouped_bar_width = (plot_width / max_points as f64).min(72.0).max(4.0) * 0.72;
    let bar_width = grouped_bar_width / spec.series.len().max(1) as f64;
    let mut scene_series = Vec::new();
    for (series_index, series) in spec.series.iter().enumerate() {
        let color = series
            .color
            .clone()
            .unwrap_or_else(|| PLOT_COLORS[series_index % PLOT_COLORS.len()].to_string());
        let mut points = Vec::new();
        let mut bars = Vec::new();
        let mut segments: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut segment: Vec<(f64, f64)> = Vec::new();
        for (index, point) in series.values.iter().enumerate() {
            match point.y.filter(|value| value.is_finite()) {
                Some(value) => {
                    let x = x_of(index);
                    let y = y_of(value);
                    let units = series.units.as_deref().unwrap_or(&spec.evidence.units);
                    let tooltip = point.label.clone().unwrap_or_else(|| {
                        format!(
                            "{} · {}: {} {}",
                            series.name,
                            point.x,
                            format_number(value),
                            units
                        )
                    });
                    points.push(ScenePoint {
                        x,
                        y,
                        tooltip: tooltip.clone(),
                    });
                    if spec.mark == PlotMark::Bar {
                        let offset = -grouped_bar_width / 2.0
                            + series_index as f64 * bar_width
                            + bar_width * 0.1;
                        bars.push(SceneBar {
                            x: x + offset,
                            y: y.min(baseline),
                            width: bar_width * 0.8,
                            height: (baseline - y).abs().max(1.0),
                            tooltip,
                        });
                    }
                    segment.push((x, y));
                }
                None => {
                    if !segment.is_empty() {
                        segments.push(std::mem::take(&mut segment));
                    }
                }
            }
        }
        if !segment.is_empty() {
            segments.push(segment);
        }

        let line_paths = segments
            .iter()
            .filter(|segment| !segment.is_empty())
            .map(|segment| series_path(segment, spec.mark))
            .collect();
        let area_paths = if spec.mark == PlotMark::Area {
            segments
                .iter()
                .filter(|segment| !segment.is_empty())
                .map(|segment| {
                    let mut path = series_path(segment, PlotMark::Line);
                    if let (Some(first), Some(last)) = (segment.first(), segment.last()) {
                        path.push_str(&format!(
                            " L {:.2} {:.2} L {:.2} {:.2} Z",
                            last.0, baseline, first.0, baseline
                        ));
                    }
                    path
                })
                .collect()
        } else {
            Vec::new()
        };
        scene_series.push(SceneSeries {
            name: series.name.clone(),
            color,
            line_paths,
            area_paths,
            points,
            bars,
        });
    }

    Ok(PlotScene {
        width,
        height,
        left,
        right,
        top,
        bottom,
        y_ticks,
        x_ticks,
        series: scene_series,
    })
}

fn series_path(points: &[(f64, f64)], mark: PlotMark) -> String {
    let Some((first_x, first_y)) = points.first() else {
        return String::new();
    };
    let mut path = format!("M {first_x:.2} {first_y:.2}");
    for (x, y) in points.iter().skip(1) {
        if mark == PlotMark::Step {
            path.push_str(&format!(" H {x:.2} V {y:.2}"));
        } else {
            path.push_str(&format!(" L {x:.2} {y:.2}"));
        }
    }
    path
}

pub fn sparkline_paths(values: &[Option<f64>], width: f64, height: f64) -> Vec<String> {
    let observed: Vec<f64> = values
        .iter()
        .filter_map(|value| *value)
        .filter(|value| value.is_finite())
        .collect();
    if observed.is_empty() {
        return Vec::new();
    }
    let mut min = observed.iter().copied().fold(f64::INFINITY, f64::min);
    let mut max = observed.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < f64::EPSILON {
        min -= 1.0;
        max += 1.0;
    }
    let count = values.len().max(1);
    let x_of = |index: usize| {
        if count <= 1 {
            width / 2.0
        } else {
            index as f64 / (count - 1) as f64 * width
        }
    };
    let y_of = |value: f64| height - ((value - min) / (max - min)) * height;
    let mut paths = Vec::new();
    let mut segment = Vec::new();
    for (index, value) in values.iter().enumerate() {
        match value.filter(|value| value.is_finite()) {
            Some(value) => segment.push((x_of(index), y_of(value))),
            None => {
                if !segment.is_empty() {
                    paths.push(series_path(&segment, PlotMark::Line));
                    segment.clear();
                }
            }
        }
    }
    if !segment.is_empty() {
        paths.push(series_path(&segment, PlotMark::Line));
    }
    paths
}

fn nice_step(raw: f64) -> f64 {
    if !raw.is_finite() || raw <= 0.0 {
        return 1.0;
    }
    let power = 10_f64.powf(raw.log10().floor());
    let fraction = raw / power;
    let nice = if fraction <= 1.0 {
        1.0
    } else if fraction <= 2.0 {
        2.0
    } else if fraction <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * power
}

fn format_number(value: f64) -> String {
    let absolute = value.abs();
    if absolute >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if absolute >= 1_000.0 {
        format!("{:.1}k", value / 1_000.0)
    } else if absolute >= 10.0 || value.fract().abs() < 0.001 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> EvidenceSpec {
        EvidenceSpec {
            question: "Is latency rising?".to_string(),
            source: "histogram/http_request_seconds".to_string(),
            window: "last 5 min".to_string(),
            freshness: "4 s".to_string(),
            units: "ms".to_string(),
            state: EvidenceState::Observed,
            reproduction: "ytrace query http-latency".to_string(),
            observed_at: None,
        }
    }

    #[test]
    fn a_plot_scene_has_scales_gaps_and_tooltips() {
        let spec = PlotSpec {
            title: "Latency".to_string(),
            subtitle: None,
            mark: PlotMark::Line,
            x_label: None,
            y_label: None,
            include_zero: false,
            height: 300,
            legend: true,
            series: vec![PlotSeries {
                name: "p95".to_string(),
                color: None,
                units: None,
                values: vec![
                    PlotPoint {
                        x: "12:00".to_string(),
                        y: Some(20.0),
                        label: None,
                    },
                    PlotPoint {
                        x: "12:01".to_string(),
                        y: None,
                        label: None,
                    },
                    PlotPoint {
                        x: "12:02".to_string(),
                        y: Some(90.0),
                        label: None,
                    },
                ],
            }],
            evidence: evidence(),
        };
        let scene = build_plot_scene(&spec).unwrap();
        assert_eq!(
            scene.series[0].line_paths.len(),
            2,
            "missing data draws a gap"
        );
        assert_eq!(scene.series[0].points.len(), 2);
        assert!(scene.series[0].points[0].tooltip.contains("p95"));
        assert!(!scene.y_ticks.is_empty());
    }

    #[test]
    fn an_emd_tree_is_bounded_and_versioned() {
        let json = serde_json::json!({
            "version": 1,
            "kind": "sparkline",
            "spec": {
                "label": "CPU",
                "values": [12.0, null, 42.0],
                "evidence": evidence()
            }
        });
        let parsed = parse_component(&json.to_string()).unwrap();
        assert!(matches!(parsed.component, EmdComponent::Sparkline(_)));

        let wrong = json.to_string().replace("\"version\":1", "\"version\":99");
        assert!(parse_component(&wrong).unwrap_err().contains("version 99"));
    }
}

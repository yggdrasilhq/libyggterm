//! The left-ruler scroll component: one mark per meaningful position on a
//! vertical ruler, click to jump.
//!
//! Owns the *rail* — geometry, tick depths, hover, click — and receives an
//! array of [`RailMark`] `{offset, depth, label}`; the HOST owns what a mark
//! means (a user turn in a transcript, a month in a photo library, a heading
//! bookmark in a document). Roadmap: *emd & notebooks* → "Demanded: the
//! left-ruler scroll component"; first host = the mini-lab testbed.
//!
//! Accessibility: marks are real buttons (Tab/Enter); labels ride `title` and
//! `aria-label`.

use dioxus::prelude::*;

/// One mark on the ruler.
#[derive(Clone, Debug, PartialEq)]
pub struct RailMark {
    /// Position along the rail as a fraction of its length, `0.0..=1.0`.
    pub offset: f32,
    /// Tick depth: `0` = long tick (major), `1` = medium, `2` = short.
    pub depth: u8,
    /// What the mark points at — the hover text and the accessible name.
    pub label: String,
}

impl RailMark {
    pub fn new(offset: f32, depth: u8, label: impl Into<String>) -> Self {
        Self { offset: clamp01(offset), depth, label: label.into() }
    }
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Colors come from the host (same pattern as `DpadPalette`); there is no
/// yggui-wide token sheet a host cannot override.
#[derive(Clone, Copy, PartialEq)]
pub struct RailPalette {
    /// The ruler edge line.
    pub track: &'static str,
    /// Mark at rest.
    pub mark: &'static str,
    /// Mark under the pointer.
    pub mark_hover: &'static str,
}

impl Default for RailPalette {
    fn default() -> Self {
        Self {
            track: "rgba(127,127,127,0.35)",
            mark: "rgba(127,127,127,0.75)",
            mark_hover: "rgba(0,0,0,0.85)",
        }
    }
}

/// Tick width and inset per depth, against the ruler edge.
fn tick(depth: u8) -> (i32, i32) {
    match depth {
        0 => (16, 4),
        1 => (10, 10),
        _ => (6, 14),
    }
}

/// The ruler. Mount beside a scrolling surface; the host scrolls it when
/// [`ScrollRailProps::on_jump`] reports the clicked mark's offset fraction.
#[component]
pub fn ScrollRail(
    /// The marks, in any order; each is clamped into `0.0..=1.0`.
    marks: Vec<RailMark>,
    palette: RailPalette,
    /// Called with the clicked mark's offset (the clamped value).
    on_jump: EventHandler<f32>,
    /// Stamped as `data-yggui-rail-surface` so probes can tell mounted rails
    /// apart and hover CSS can be scoped per surface.
    #[props(default = String::new())]
    surface_id: String,
) -> Element {
    let scope = if surface_id.is_empty() { "default".to_string() } else { surface_id.clone() };
    let hover_css = format!(
        "[data-yggui-rail-surface='{scope}'] .yggui-rail-mark:hover {{ \
         opacity:1 !important; background:{} !important; }}",
        palette.mark_hover
    );
    let key_style = format!(
        "position:absolute; height:2px; border:none; border-radius:1px; \
         padding:0; cursor:pointer; background:{}; opacity:0.7;",
        palette.mark
    );
    let rendered: Vec<(f32, String, u8, String)> = marks
        .iter()
        .map(|mark| {
            let (width, left) = tick(mark.depth);
            (
                clamp01(mark.offset),
                mark.label.clone(),
                mark.depth,
                format!(
                    "{key_style} top:calc({} * 100% - 1px); width:{width}px; left:{left}px;",
                    clamp01(mark.offset)
                ),
            )
        })
        .collect();
    rsx! {
        style { "{hover_css}" }
        div {
            "data-yggui-rail-surface": "{scope}",
            style: format!(
                "position:relative; width:24px; align-self:stretch; flex:none; \
                 border-right:1px solid {};",
                palette.track
            ),
            for (i, (offset, label, depth, style)) in rendered.into_iter().enumerate() {
                button {
                    key: "{i}-{offset}",
                    class: "yggui-rail-mark",
                    r#type: "button",
                    "data-yggui-rail-mark": "{depth}",
                    title: "{label}",
                    "aria-label": "{label}",
                    style: "{style}",
                    onclick: move |evt| {
                        evt.prevent_default();
                        evt.stop_propagation();
                        on_jump.call(offset);
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_offsets() {
        assert_eq!(clamp01(-0.2), 0.0);
        assert_eq!(clamp01(0.42), 0.42);
        assert_eq!(clamp01(1.3), 1.0);
    }

    #[test]
    fn rail_mark_new_clamps() {
        let m = RailMark::new(1.7, 1, "late");
        assert_eq!(m.offset, 1.0);
        assert_eq!(m.label, "late");
    }

    #[test]
    fn depth_ticks_shrink() {
        let (w0, _) = tick(0);
        let (w1, _) = tick(1);
        let (w2, _) = tick(2);
        assert!(w0 > w1 && w1 > w2);
    }

    #[test]
    fn unknown_depth_is_the_short_tick() {
        let (w3, _) = tick(3);
        let (w2, _) = tick(2);
        assert_eq!(w3, w2);
    }
}

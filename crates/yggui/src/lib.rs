// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![recursion_limit = "256"]

pub mod chat_input;
/// Real window controls, which need a real window. The ONE module bound to a
/// platform; everything else here is plain `rsx!` and travels anywhere Dioxus
/// does.
#[cfg(feature = "desktop-shell")]
pub mod chrome;
pub mod conversation;
pub mod drag_tree;
pub mod drag_visuals;
pub mod motion;
pub mod notifications;
pub mod otp;
pub mod rails;
pub mod theme;

pub use chat_input::{CHAT_INPUT_CSS, ChatContextOption, ComposerSendShortcut, YggChatInputBox};
#[cfg(feature = "desktop-shell")]
pub use chrome::{
    ChromeControlIcon, ChromePalette, HoveredChromeControl, TitlebarChrome, WindowControlsStrip,
    search_field_shell_style, search_input_style,
};
pub use conversation::{
    AssistantTurn, CHANGED_FILE_CHIP_LIMIT, CONVERSATION_COLUMN_PX, CONVERSATION_CSS,
    CONVERSATION_THEME_CSS, ChangedFileChips, ConversationColumn, ConversationEmptyState,
    ConversationTokens, DiffStat, QuietButton, SystemTurn, TurnAction, TurnDivider, UserTurn,
    WORK_GROUP_COLLAPSED_ROWS, WorkGroup, WorkMark, WorkMarkGlyph, WorkRow, WorkingIndicator,
    changed_file_label,
};
pub use drag_tree::{
    DRAG_BEGIN_THRESHOLD_PX, DragDropPlacement, DragDropTarget, ROW_DRAG_CLICK_SUPPRESS_MS,
    ROW_DRAG_SPRING_MS, RowDragGesture, RowDragHover, RowDropTarget, RowTreeDrop, RowTreeRow,
    TreeDropPlacement, TreeReorderItem, TreeReorderPlanItem, build_tree_reorder_plan,
    canonical_tree_leaf_name, drag_threshold_reached, join_tree_child_path,
    ordered_tree_child_path, reorder_row_tree, resolve_drag_drop_target,
    resolve_tree_drop_placement, row_tree_descends_from, staging_tree_child_path, tree_parent_path,
    tree_path_contains, valid_drop_target,
};
pub use drag_visuals::{DragGhostCard, DragGhostPalette, TreeDropZones};
pub use motion::{
    MOTION_EMPHASIZED_DECELERATE, MOTION_ENTER_DURATION_MS, emphasized_enter_transition,
    emphasized_exit_transition, standard_accelerate_transition, standard_decelerate_transition,
    standard_transition, transition,
};
pub use notifications::{
    TOAST_CSS, ToastAnchor, ToastCard, ToastItem, ToastPalette, ToastTone, ToastViewport,
};
pub use otp::{
    OtpCodeEntry, YGGUI_OTP_CODE_LEN, YGGUI_OTP_CSS, complete_otp, digits_for_otp,
    install_otp_paste_bridge_script, otp_paste_from_native_script,
};
pub use rails::{RailHeader, RailScrollBody, RailSectionTitle, SideRailReveal, SideRailShell};
pub use theme::{
    MAX_THEME_STOPS, THEME_EDITOR_SWATCHES, append_theme_stop, chrome_material_tint,
    clamp_theme_spec, default_theme_editor_spec, dominant_accent, gradient_background_repeat_css,
    gradient_background_size_css, gradient_css, hex_to_rgb, live_blur_gradient_css,
    material_blur_radius_px, preview_surface_css, shell_tint,
};

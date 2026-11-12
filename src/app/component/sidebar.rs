use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::*;

use crate::herdr::{Herdr, Workspace};

pub struct Sidebar {
    workspaces: Vec<Workspace>,
}

impl Sidebar {
    pub fn new(_: &mut Context<Self>) -> Self {
        let workspaces = Herdr::new().list_workspaces().unwrap_or_default();
        Self { workspaces }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let entries = self.workspaces.iter().map(|workspace| {
            let row = div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .p_2()
                .rounded_md()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(workspace.label.clone()),
                        )
                        .child(div().text_sm().child("Created 2 hours ago")),
                )
                .child(
                    Button::new(format!("workspace-menu-{}", workspace.workspace_id))
                        .ghost()
                        .label("⋯")
                        .cursor_pointer(),
                );

            if workspace.focused {
                row.bg(gpui_kit::blue().opacity(0.15))
            } else {
                row
            }
        });

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .w_full()
            .child(div().font_weight(FontWeight::BOLD).text_lg().child("Tasks"))
            .children(entries)
    }
}

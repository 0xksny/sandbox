use gpui_kit::*;

use crate::app::component::chat::Chat;
use crate::app::component::sidebar::Sidebar;

pub struct App {
    sidebar: Entity<Sidebar>,
    chat: Entity<Chat>,
}

impl App {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            sidebar: cx.new(Sidebar::new),
            chat: cx.new(Chat::new),
        }
    }
}

impl Render for App {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(div().w(px(240.)).h_full().child(self.sidebar.clone()))
            .child(div().flex_1().h_full().child(self.chat.clone()))
    }
}

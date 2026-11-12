use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Textarea, TextareaState};
use gpui_kit::component::{Icon, IconName};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub struct Chat {
    messages: Vec<String>,
    waiting: bool,
}

impl Chat {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            messages: vec!["Hi! What would you like to work on?".into()],
            waiting: false,
        }
    }
}

impl Render for Chat {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let draft = window.use_state(cx, |window, cx| {
            TextareaState::new(window, cx)
                .auto_grow(2, 6)
                .submit_on_enter(true)
                .placeholder("Message…")
        });
        let draft_for_send = draft.clone();
        let messages = self.messages.iter().map(|message| {
            div()
                .w_full()
                .p_3()
                .rounded_md()
                .border_1()
                .child(message.clone())
        });

        let send = cx.listener(move |this, _, window, cx| {
            let message = draft_for_send.read(cx).value().trim().to_string();
            if message.is_empty() || this.waiting {
                return;
            }
            this.messages.push(message);
            this.waiting = true;
            draft_for_send.update(cx, |state, cx| state.set_value("", window, cx));
            cx.notify();
            let weak = cx.entity().downgrade();
            cx.spawn(async move |_, cx| {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                let _ = weak.update(cx, |this, cx| {
                    this.messages
                        .push("Thanks for sharing that. Let me think it through with you.".into());
                    this.waiting = false;
                    cx.notify();
                });
            })
            .detach();
        });

        let on_key_down = cx.listener({
            let draft = draft.clone();
            move |this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "enter" && !event.keystroke.modifiers.shift {
                    let message = draft.read(cx).value().trim().to_string();
                    if !message.is_empty() && !this.waiting {
                        this.messages.push(message);
                        this.waiting = true;
                        draft.update(cx, |state, cx| state.set_value("", window, cx));
                        let weak = cx.entity().downgrade();
                        cx.spawn(async move |_, cx| {
                            cx.background_executor()
                                .timer(std::time::Duration::from_secs(1))
                                .await;
                            let _ = weak.update(cx, |this, cx| {
                                this.messages.push(
                                    "Thanks for sharing that. Let me think it through with you."
                                        .into(),
                                );
                                this.waiting = false;
                                cx.notify();
                            });
                        })
                        .detach();
                        cx.notify();
                    }
                    cx.stop_propagation();
                }
            }
        });

        div()
            .flex()
            .flex_col()
            .size_full()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .children(messages)
                    .when(self.waiting, |el| el.child("Thinking…")),
            )
            .child(
                div().w_full().p_4().flex().flex_col().gap_2().child(
                    div()
                        .relative()
                        .w_full()
                        .on_key_down(on_key_down)
                        .child(Textarea::new(&draft).w_full().pr_12())
                        .child(
                            div().absolute().right_2().bottom_2().child(
                                Button::new("send-message")
                                    .primary()
                                    .rounded_full()
                                    .p_2()
                                    .icon(Icon::new(IconName::ArrowUp).small())
                                    .on_click(send),
                            ),
                        ),
                ),
            )
    }
}

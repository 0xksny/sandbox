use gpui_kit::*;

use crate::app::component::app::App as AppComponent;

mod component;

actions!(app, [Quit]);

pub struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub fn run(self) -> Result<()> {
        application().with_assets(assets::Assets).run(|cx| {
            init(cx);
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
            cx.set_menus(vec![Menu {
                name: "Sandbox".into(),
                items: vec![MenuItem::action("Quit", Quit)],
                disabled: false,
            }]);
            #[allow(clippy::expect_used)]
            open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(AppComponent::new)
            })
            .expect("failed to open window");
        });
        Ok(())
    }
}

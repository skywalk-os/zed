mod nix_commands;

pub use nix_commands::{current_generation, list_generations, Generation};

use gpui::{
    actions, div, px, Action, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ParentElement, Pixels, Render, Styled, WeakEntity, Window,
};
use serde::Deserialize;
use ui::{prelude::*, IconName, Label, LabelCommon, LabelSize};
use workspace::{
    dock::{DockPosition, Panel, PanelEvent},
    Workspace,
};

actions!(nix_generations_panel, [ToggleFocus]);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &ToggleFocus, window, cx| {
            workspace.toggle_panel_focus::<NixGenerationsPanel>(window, cx);
        });
    })
}

pub struct NixGenerationsPanel {
    position: DockPosition,
    focus_handle: FocusHandle,
    generations: Vec<Generation>,
    _workspace: WeakEntity<Workspace>,
}

impl NixGenerationsPanel {
    pub fn new(workspace: &Workspace, cx: &mut Context<Self>) -> Self {
        Self {
            position: DockPosition::Left,
            focus_handle: cx.focus_handle(),
            generations: Vec::new(),
            _workspace: workspace.weak_handle(),
        }
    }

    pub fn load(
        workspace: WeakEntity<Workspace>,
        cx: gpui::AsyncWindowContext,
    ) -> gpui::Task<anyhow::Result<Entity<Self>>> {
        cx.spawn(async move |mut cx| {
            workspace.update_in(&mut cx, |workspace, window, cx| {
                cx.new(|cx| Self::new(workspace, cx))
            })
        })
    }

    fn refresh_generations(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, mut cx| {
            let generations = list_generations().await.unwrap_or_default();
            this.update_in(&mut cx, |this, _window, cx| {
                this.generations = generations;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Focusable for NixGenerationsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for NixGenerationsPanel {}

impl Panel for NixGenerationsPanel {
    fn persistent_name() -> &'static str {
        "NixGenerationsPanel"
    }

    fn panel_key() -> &'static str {
        "NixGenerationsPanel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        self.position
    }

    fn position_is_valid(&self, _position: DockPosition) -> bool {
        true
    }

    fn set_position(&mut self, position: DockPosition, _window: &mut Window, cx: &mut Context<Self>) {
        self.position = position;
        cx.notify();
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        px(320.)
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        Some(IconName::ListTree)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("NixOS Generations")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleFocus)
    }

    fn activation_priority(&self) -> u32 {
        200
    }
}

impl Render for NixGenerationsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let generations = &self.generations;

        div()
            .key_context("NixGenerationsPanel")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .child(
                div()
                    .p_2()
                    .child(
                        Label::new("NixOS Generations")
                            .size(LabelSize::Large)
                            .color(Color::Default),
                    ),
            )
            .children(generations.iter().map(|gen| {
                let is_current = gen.current;
                div()
                    .px_2()
                    .py_1()
                    .when(is_current, |el| el.bg(cx.theme().colors().ghost_element_selected))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(Label::new(format!("#{}", gen.number)).size(LabelSize::Default))
                            .child(Label::new(gen.date.clone()).size(LabelSize::Small).color(Color::Muted))
                            .when(is_current, |el| {
                                el.child(Label::new("(current)").size(LabelSize::Small).color(Color::Accent))
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(Label::new(format!("NixOS {}", gen.nixos_version)).size(LabelSize::Small).color(Color::Muted))
                            .child(Label::new(format!("kernel {}", gen.kernel_version)).size(LabelSize::Small).color(Color::Muted)),
                    )
            }))
    }
}

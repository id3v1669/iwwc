use std::time::Duration;

use crate::config::primitives::Transition;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, mouse, overlay, renderer};
use iced::animation::{Animation, Easing};
use iced::time::Instant;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, window};

fn revealed_geometry(transition: Transition, full: Size, factor: f32) -> (Size, Point) {
    match transition {
        Transition::None => (
            Size::new(full.width * factor, full.height * factor),
            Point::ORIGIN,
        ),
        Transition::SlideLeft => (Size::new(full.width * factor, full.height), Point::ORIGIN),
        Transition::SlideRight => {
            let w = full.width * factor;
            (Size::new(w, full.height), Point::new(w - full.width, 0.0))
        }
        Transition::SlideUp => (Size::new(full.width, full.height * factor), Point::ORIGIN),
        Transition::SlideDown => {
            let h = full.height * factor;
            (Size::new(full.width, h), Point::new(0.0, h - full.height))
        }
    }
}

// #rmstatic1
const EASING: Easing = Easing::EaseInOut;

pub struct Revealer<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    active: bool,
    transition: Transition,
    duration: Duration,
}

struct State {
    animation: Animation<bool>,
    duration: Duration,
    now: Instant,
}

impl<'a, Message, Theme, Renderer> Revealer<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        active: bool,
        transition: Transition,
        duration: Duration,
    ) -> Self {
        Self {
            content: content.into(),
            active,
            transition,
            duration,
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Revealer<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            animation: Animation::new(self.active)
                .duration(self.duration)
                .easing(EASING),
            duration: self.duration,
            now: Instant::now(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        if state.duration != self.duration {
            state.animation = Animation::new(state.animation.value())
                .duration(self.duration)
                .easing(EASING);
            state.duration = self.duration;
        }
        if state.animation.value() != self.active {
            if self.transition == Transition::None {
                state.animation = Animation::new(self.active)
                    .duration(self.duration)
                    .easing(EASING);
            } else {
                state.animation.go_mut(self.active, Instant::now());
            }
        }
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let factor = {
            let state = tree.state.downcast_ref::<State>();
            state.animation.interpolate(0.0, 1.0, state.now)
        };
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let full = child.size();
        let (size, offset) = revealed_geometry(self.transition, full, factor);
        layout::Node::with_children(size, vec![child.move_to(offset)])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let masked_cursor = if cursor.is_over(layout.bounds()) {
            cursor
        } else {
            mouse::Cursor::Unavailable
        };
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            masked_cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<State>();
            let was_animating = state.animation.is_animating(state.now);
            state.now = *now;
            let is_animating = state.animation.is_animating(*now);
            if was_animating || is_animating {
                shell.invalidate_layout();
            }
            if is_animating {
                shell.request_redraw();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::default();
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let Some(clipped_viewport) = bounds.intersection(viewport) else {
            // fully collapsed (or offscreen): nothing to draw
            return;
        };
        let state = tree.state.downcast_ref::<State>();
        let revealing = state.animation.is_animating(state.now);
        let child_layout = layout.children().next().unwrap();
        if revealing {
            renderer.with_layer(clipped_viewport, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    child_layout,
                    cursor,
                    &clipped_viewport,
                );
            });
        } else {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Revealer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(revealer: Revealer<'a, Message, Theme, Renderer>) -> Self {
        Element::new(revealer)
    }
}

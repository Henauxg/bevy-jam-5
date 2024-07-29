//! Helper traits for creating common widgets.

use bevy::{ecs::system::EntityCommands, prelude::*, ui::Val::*};

use super::{interaction::InteractionPalette, palette::*};

/// An extension trait for spawning UI widgets.
pub trait Widgets {
    /// Spawn a simple button with text.
    fn button(&mut self, text: impl Into<String>, font: Handle<Font>) -> EntityCommands;

    /// Spawn a simple header label. Bigger than [`Widgets::label`].
    fn header(&mut self, text: impl Into<String>) -> EntityCommands;

    /// Spawn a simple text label.
    fn label(&mut self, text: impl Into<String>) -> EntityCommands;

    /// Spawn a simple text label.
    fn dynamic_label_with_marker<C: Component>(
        &mut self,
        title: impl Into<String>,
        text: impl Into<String>,
        marker: C,
        font: Handle<Font>,
    ) -> EntityCommands;
}

impl<T: Spawn> Widgets for T {
    fn button(&mut self, text: impl Into<String>, font: Handle<Font>) -> EntityCommands {
        let mut entity = self.spawn((
            Name::new("Button"),
            ButtonBundle {
                style: Style {
                    width: Px(170.0),
                    height: Px(55.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(NODE_BACKGROUND_COLOR),
                border_radius: BorderRadius::all(Val::Px(5.)),
                ..default()
            },
            InteractionPalette {
                none: NODE_BACKGROUND_COLOR,
                hovered: BUTTON_HOVERED_BACKGROUND,
                pressed: BUTTON_PRESSED_BACKGROUND,
            },
        ));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Button Text"),
                TextBundle::from_section(
                    text,
                    TextStyle {
                        font_size: 40.0,
                        color: BUTTON_TEXT_COLOR,
                        font,
                        ..default()
                    },
                ),
            ));
        });
        entity
    }

    fn header(&mut self, text: impl Into<String>) -> EntityCommands {
        let mut entity = self.spawn((
            Name::new("Header"),
            NodeBundle {
                style: Style {
                    width: Px(500.0),
                    height: Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(NODE_BACKGROUND_COLOR),
                ..default()
            },
        ));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Header Text"),
                TextBundle::from_section(
                    text,
                    TextStyle {
                        font_size: 40.0,
                        color: HEADER_TEXT_COLOR,
                        ..default()
                    },
                ),
            ));
        });
        entity
    }

    fn label(&mut self, text: impl Into<String>) -> EntityCommands {
        let mut entity = self.spawn((
            Name::new("Label"),
            NodeBundle {
                style: Style {
                    width: Px(500.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Label Text"),
                TextBundle::from_section(
                    text,
                    TextStyle {
                        font_size: LABEL_SIZE,
                        color: LABEL_TEXT_COLOR_2,
                        ..default()
                    },
                ),
            ));
        });
        entity
    }

    fn dynamic_label_with_marker<C: Component>(
        &mut self,
        title: impl Into<String>,
        text: impl Into<String>,
        marker: C,
        font: Handle<Font>,
    ) -> EntityCommands {
        let mut entity = self.spawn((
            Name::new("Label"),
            NodeBundle {
                style: Style {
                    width: Px(500.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Label Text"),
                TextBundle::from_sections([
                    TextSection {
                        value: title.into(),
                        style: TextStyle {
                            font_size: LABEL_SIZE,
                            color: LABEL_TEXT_COLOR_2,
                            font: font.clone_weak(),
                            ..default()
                        },
                    },
                    TextSection {
                        value: text.into(),
                        style: TextStyle {
                            font_size: LABEL_SIZE,
                            color: LABEL_TEXT_COLOR_2,
                            font: font.clone_weak(),
                            ..default()
                        },
                    },
                ]),
                marker,
            ));
        });
        entity
    }
}

/// An extension trait for spawning UI containers.
pub trait Containers {
    /// Spawns a root node that covers the full screen
    /// and centers its content horizontally and vertically.
    fn ui_root(&mut self) -> EntityCommands;

    fn bottom_ui_root(&mut self) -> EntityCommands;
    fn bottom_left_ui_root(&mut self) -> EntityCommands;
    fn top_ui_root(&mut self) -> EntityCommands;
}

impl Containers for Commands<'_, '_> {
    fn ui_root(&mut self) -> EntityCommands {
        self.spawn((
            Name::new("UI Root"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
    }

    fn bottom_ui_root(&mut self) -> EntityCommands {
        self.spawn((
            Name::new("Bottom UI Root"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(98.0),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
    }

    fn bottom_left_ui_root(&mut self) -> EntityCommands {
        self.spawn((
            Name::new("Bottom left UI Root"),
            NodeBundle {
                style: Style {
                    width: Percent(35.0),
                    height: Percent(20.0),
                    bottom: Val::Percent(2.),
                    left: Val::Percent(2.),
                    align_content: bevy::ui::AlignContent::Center,
                    justify_content: bevy::ui::JustifyContent::Center,
                    // justify_content: JustifyContent::Start,
                    // align_items: AlignItems::End,
                    // flex_direction: FlexDirection::Row,
                    // row_gap: Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
    }

    fn top_ui_root(&mut self) -> EntityCommands {
        self.spawn((
            Name::new("Top UI Root"),
            NodeBundle {
                style: Style {
                    top: Val::Percent(2.),
                    width: Percent(100.0),
                    height: Percent(95.0),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
    }
}

/// An internal trait for types that can spawn entities.
/// This is here so that [`Widgets`] can be implemented on all types that
/// are able to spawn entities.
/// Ideally, this trait should be [part of Bevy itself](https://github.com/bevyengine/bevy/issues/14231).
trait Spawn {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands;
}

impl Spawn for Commands<'_, '_> {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        self.spawn(bundle)
    }
}

impl Spawn for ChildBuilder<'_> {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        self.spawn(bundle)
    }
}

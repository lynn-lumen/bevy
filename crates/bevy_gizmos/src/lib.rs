#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(
    html_logo_url = "https://bevyengine.org/assets/icon.png",
    html_favicon_url = "https://bevyengine.org/assets/icon.png"
)]

//! This crate adds an immediate mode drawing api to Bevy for visual debugging.
//!
//! # Example
//! ```
//! # use bevy_gizmos::prelude::*;
//! # use bevy_render::prelude::*;
//! # use bevy_math::prelude::*;
//! # use bevy_color::palettes::basic::GREEN;
//! fn system(mut gizmos: Gizmos) {
//!     gizmos.line(Vec3::ZERO, Vec3::X, GREEN);
//! }
//! # bevy_ecs::system::assert_is_system(system);
//! ```
//!
//! See the documentation on [Gizmos](crate::gizmos::Gizmos) for more examples.

/// System set label for the systems handling the rendering of gizmos.
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GizmoRenderSystem {
    /// Adds gizmos to the [`Transparent2d`](bevy_core_pipeline::core_2d::Transparent2d) render phase
    #[cfg(feature = "bevy_sprite")]
    QueueLineGizmos2d,
    /// Adds gizmos to the [`Transparent3d`](bevy_core_pipeline::core_3d::Transparent3d) render phase
    #[cfg(feature = "bevy_pbr")]
    QueueLineGizmos3d,
}

pub mod aabb;
pub mod arcs;
pub mod arrows;
pub mod circles;
pub mod config;
pub mod gizmos;
pub mod grid;
pub mod light;
pub mod primitives;

mod lines;

/// The `bevy_gizmos` prelude.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        aabb::{AabbGizmoConfigGroup, ShowAabbGizmo},
        config::{
            DefaultGizmoConfigGroup, GizmoConfig, GizmoConfigGroup, GizmoConfigStore,
            GizmoLineJoint, GizmoLineStyle,
        },
        gizmos::Gizmos,
        light::{LightGizmoColor, LightGizmoConfigGroup, ShowLightGizmo},
        primitives::{dim2::GizmoPrimitive2d, dim3::GizmoPrimitive3d},
        AppGizmoBuilder,
    };
}

use aabb::AabbGizmoPlugin;
use bevy_app::{App, Plugin};
use bevy_ecs::schedule::SystemSet;
use config::{DefaultGizmoConfigGroup, GizmoConfig, GizmoConfigGroup, GizmoConfigStore};
use gizmos::GizmoStorage;
use light::LightGizmoPlugin;
use lines::{init_line_gizmo_group, LineGizmoPlugin};

/// A [`Plugin`] that provides an immediate mode drawing api for visual debugging.
pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Gizmos cannot work without either a 3D or 2D renderer.
        #[cfg(all(not(feature = "bevy_pbr"), not(feature = "bevy_sprite")))]
        bevy_utils::tracing::error!(
            "bevy_gizmos requires either bevy_pbr or bevy_sprite. Please enable one."
        );

        app.register_type::<GizmoConfig>()
            .register_type::<GizmoConfigStore>()
            // We insert the Resource GizmoConfigStore into the world implicitly here if it does not exist.
            .init_gizmo_group::<DefaultGizmoConfigGroup>()
            .add_plugins(AabbGizmoPlugin)
            .add_plugins(LightGizmoPlugin)
            .add_plugins(LineGizmoPlugin);
    }
}

/// A trait adding `init_gizmo_group<T>()` to the app
pub trait AppGizmoBuilder {
    /// Registers [`GizmoConfigGroup`] `T` in the app enabling the use of [Gizmos&lt;T&gt;](crate::gizmos::Gizmos).
    ///
    /// Configurations can be set using the [`GizmoConfigStore`] [`Resource`].
    fn init_gizmo_group<T: GizmoConfigGroup + Default>(&mut self) -> &mut Self;

    /// Insert the [`GizmoConfigGroup`] in the app with the given value and [`GizmoConfig`].
    ///
    /// This method should be preferred over [`AppGizmoBuilder::init_gizmo_group`] if and only if you need to configure fields upon initialization.
    fn insert_gizmo_group<T: GizmoConfigGroup>(
        &mut self,
        group: T,
        config: GizmoConfig,
    ) -> &mut Self;
}

impl AppGizmoBuilder for App {
    fn init_gizmo_group<T: GizmoConfigGroup + Default>(&mut self) -> &mut Self {
        if self.world.contains_resource::<GizmoStorage<T>>() {
            return self;
        }

        init_line_gizmo_group::<T>(self);

        self.world
            .get_resource_or_insert_with::<GizmoConfigStore>(Default::default)
            .register::<T>();

        self
    }

    fn insert_gizmo_group<T: GizmoConfigGroup>(
        &mut self,
        group: T,
        config: GizmoConfig,
    ) -> &mut Self {
        self.world
            .get_resource_or_insert_with::<GizmoConfigStore>(Default::default)
            .insert(config, group);

        if self.world.contains_resource::<GizmoStorage<T>>() {
            return self;
        }

        init_line_gizmo_group::<T>(self);

        self
    }
}

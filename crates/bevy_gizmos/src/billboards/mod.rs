use crate::config::{GizmoConfigGroup, GizmoConfigStore, GizmoMeshConfig};
use crate::gizmos::GizmoStorage;
use bevy_app::{App, Last, Plugin};
use bevy_asset::{load_internal_asset, Asset, AssetApp, Assets, Handle};
use bevy_color::LinearRgba;
use bevy_ecs::{
    component::Component,
    query::ROQueryItem,
    schedule::IntoSystemConfigs,
    system::{
        lifetimeless::{Read, SRes},
        Commands, Res, ResMut, Resource, SystemParamItem,
    },
};
use bevy_math::{Vec2, Vec3};
use bevy_reflect::TypePath;
use bevy_render::{
    extract_component::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
    render_asset::{
        PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssetUsages, RenderAssets,
    },
    render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
    render_resource::{
        binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
        BindGroupLayoutEntries, Buffer, BufferInitDescriptor, BufferUsages, Shader, ShaderStages,
        ShaderType, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
    },
    renderer::RenderDevice,
    Extract, ExtractSchedule, Render, RenderApp, RenderSet,
};
use bevy_utils::TypeIdMap;
use bytemuck::cast_slice;
use std::{any::TypeId, mem};

#[cfg(feature = "bevy_sprite")]
mod pipeline_2d;
#[cfg(feature = "bevy_pbr")]
mod pipeline_3d;

const BILLBOARD_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(7414812689238026510);

pub struct BillboardGizmoPlugin;
impl Plugin for BillboardGizmoPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Gizmos cannot work without either a 3D or 2D renderer.
        #[cfg(all(not(feature = "bevy_pbr"), not(feature = "bevy_sprite")))]
        bevy_utils::tracing::error!(
            "bevy_gizmos requires either bevy_pbr or bevy_sprite. Please enable one."
        );

        load_internal_asset!(
            app,
            BILLBOARD_SHADER_HANDLE,
            "billboards.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(UniformComponentPlugin::<BillboardGizmoUniform>::default())
            .init_asset::<BillboardGizmo>()
            .add_plugins(RenderAssetPlugin::<BillboardGizmo>::default())
            .init_resource::<BillboardGizmoHandles>();

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            Render,
            prepare_billboard_gizmo_bind_group.in_set(RenderSet::PrepareBindGroups),
        );

        render_app.add_systems(ExtractSchedule, extract_gizmo_data);

        #[cfg(feature = "bevy_sprite")]
        app.add_plugins(pipeline_2d::BillboardGizmo2dPlugin);
        #[cfg(feature = "bevy_pbr")]
        app.add_plugins(pipeline_3d::BillboardGizmo3dPlugin);
    }

    fn finish(&self, app: &mut bevy_app::App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let render_device = render_app.world.resource::<RenderDevice>();
        let billboard_layout = render_device.create_bind_group_layout(
            "BillboardGizmoUniform layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                uniform_buffer::<BillboardGizmoUniform>(true),
            ),
        );

        render_app.insert_resource(BillboardGizmoUniformBindgroupLayout {
            layout: billboard_layout,
        });
    }
}

pub(crate) fn init_billboard_gizmo_group<T: GizmoConfigGroup + Default>(app: &mut App) {
    let mut handles = app
        .world
        .get_resource_or_insert_with::<BillboardGizmoHandles>(Default::default);
    handles.billboard.insert(TypeId::of::<T>(), None);

    app.init_resource::<GizmoStorage<T>>()
        .add_systems(Last, update_gizmo_meshes::<T>);
}

/// Holds handles to the billboard gizmos for each gizmo configuration group
// As `TypeIdMap` iteration order depends on the order of insertions and deletions, this uses
// `Option<Handle>` to be able to reserve the slot when creating the gizmo configuration group.
// That way iteration order is stable across executions and depends on the order of configuration
// group creation.
#[derive(Resource, Default)]
struct BillboardGizmoHandles {
    billboard: TypeIdMap<Option<Handle<BillboardGizmo>>>,
}

fn update_gizmo_meshes<T: GizmoConfigGroup>(
    mut billboard_gizmos: ResMut<Assets<BillboardGizmo>>,
    mut handles: ResMut<BillboardGizmoHandles>,
    mut storage: ResMut<GizmoStorage<T>>,
    config_store: Res<GizmoConfigStore>,
) {
    if storage.billboard_positions.is_empty() {
        handles.billboard.insert(TypeId::of::<T>(), None);
    } else if let Some(handle) = handles.billboard.get_mut(&TypeId::of::<T>()) {
        if let Some(handle) = handle {
            let billboard = billboard_gizmos.get_mut(handle.id()).unwrap();

            billboard.positions = mem::take(&mut storage.billboard_positions);
            billboard.colors = mem::take(&mut storage.billboard_colors);
        } else {
            let mut billboard = BillboardGizmo::default();

            billboard.positions = mem::take(&mut storage.billboard_positions);
            billboard.colors = mem::take(&mut storage.billboard_colors);

            *handle = Some(billboard_gizmos.add(billboard));
        }
    }
}

fn extract_gizmo_data(
    mut commands: Commands,
    handles: Extract<Res<BillboardGizmoHandles>>,
    config: Extract<Res<GizmoConfigStore>>,
) {
    for (group_type_id, handle) in handles.billboard.iter() {
        let Some((config, _)) = config.get_config_dyn(group_type_id) else {
            continue;
        };

        if !config.enabled {
            continue;
        }

        let Some(handle) = handle else {
            continue;
        };

        commands.spawn((
            BillboardGizmoUniform {
                billboard_size: config.billboard_size,
                depth_bias: config.depth_bias,
                #[cfg(feature = "webgl")]
                _padding: Default::default(),
            },
            (*handle).clone_weak(),
            GizmoMeshConfig::from(config),
        ));
    }
}

#[derive(Component, ShaderType, Clone, Copy)]
struct BillboardGizmoUniform {
    billboard_size: Vec2,
    depth_bias: f32,
    /// WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl")]
    _padding: f32,
}

#[derive(Asset, Debug, Default, Clone, TypePath)]
struct BillboardGizmo {
    positions: Vec<Vec3>,
    colors: Vec<LinearRgba>,
}

#[derive(Debug, Clone)]
struct GpuBillboardGizmo {
    position_buffer: Buffer,
    color_buffer: Buffer,
    vertex_count: u32,
}

impl RenderAsset for BillboardGizmo {
    type PreparedAsset = GpuBillboardGizmo;
    type Param = SRes<RenderDevice>;

    fn asset_usage(&self) -> RenderAssetUsages {
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD
    }

    fn prepare_asset(
        self,
        render_device: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self>> {
        let position_buffer_data = cast_slice(&self.positions);
        let position_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: Some("BillboardGizmo Position Buffer"),
            contents: position_buffer_data,
        });

        let color_buffer_data = cast_slice(&self.colors);
        let color_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: Some("BillboardGizmo Color Buffer"),
            contents: color_buffer_data,
        });

        Ok(GpuBillboardGizmo {
            position_buffer,
            color_buffer,
            vertex_count: self.positions.len() as u32,
        })
    }
}

#[derive(Resource)]
struct BillboardGizmoUniformBindgroupLayout {
    layout: BindGroupLayout,
}

#[derive(Resource)]
struct BillboardGizmoUniformBindgroup {
    bindgroup: BindGroup,
}

fn prepare_billboard_gizmo_bind_group(
    mut commands: Commands,
    billboard_gizmo_uniform_layout: Res<BillboardGizmoUniformBindgroupLayout>,
    render_device: Res<RenderDevice>,
    billboard_gizmo_uniforms: Res<ComponentUniforms<BillboardGizmoUniform>>,
) {
    if let Some(binding) = billboard_gizmo_uniforms.uniforms().binding() {
        commands.insert_resource(BillboardGizmoUniformBindgroup {
            bindgroup: render_device.create_bind_group(
                "BillboardGizmoUniform bindgroup",
                &billboard_gizmo_uniform_layout.layout,
                &BindGroupEntries::single(binding),
            ),
        });
    }
}

struct SetBillboardGizmoBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetBillboardGizmoBindGroup<I> {
    type Param = SRes<BillboardGizmoUniformBindgroup>;
    type ViewQuery = ();
    type ItemQuery = Read<DynamicUniformIndex<BillboardGizmoUniform>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        uniform_index: Option<ROQueryItem<'w, Self::ItemQuery>>,
        bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(uniform_index) = uniform_index else {
            return RenderCommandResult::Failure;
        };
        pass.set_bind_group(
            I,
            &bind_group.into_inner().bindgroup,
            &[uniform_index.index()],
        );
        RenderCommandResult::Success
    }
}

struct DrawBillboardGizmo;
impl<P: PhaseItem> RenderCommand<P> for DrawBillboardGizmo {
    type Param = SRes<RenderAssets<BillboardGizmo>>;
    type ViewQuery = ();
    type ItemQuery = Read<Handle<BillboardGizmo>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        handle: Option<ROQueryItem<'w, Self::ItemQuery>>,
        billboard_gizmos: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(handle) = handle else {
            return RenderCommandResult::Failure;
        };
        let Some(billboard_gizmo) = billboard_gizmos.into_inner().get(handle) else {
            return RenderCommandResult::Failure;
        };

        if billboard_gizmo.vertex_count == 0 {
            return RenderCommandResult::Success;
        }

        let instances = {
            pass.set_vertex_buffer(0, billboard_gizmo.position_buffer.slice(..));
            pass.set_vertex_buffer(1, billboard_gizmo.color_buffer.slice(..));

            billboard_gizmo.vertex_count
        };

        pass.draw(0..6, 0..instances);

        RenderCommandResult::Success
    }
}

fn billboard_gizmo_vertex_buffer_layouts() -> Vec<VertexBufferLayout> {
    use VertexFormat::*;
    let position_layout = VertexBufferLayout {
        array_stride: Float32x3.size(),
        step_mode: VertexStepMode::Instance,
        attributes: vec![VertexAttribute {
            format: Float32x3,
            offset: 0,
            shader_location: 0,
        }],
    };

    let color_layout = VertexBufferLayout {
        array_stride: Float32x4.size(),
        step_mode: VertexStepMode::Instance,
        attributes: vec![VertexAttribute {
            format: Float32x4,
            offset: 0,
            shader_location: 1,
        }],
    };
    vec![position_layout, color_layout]
}

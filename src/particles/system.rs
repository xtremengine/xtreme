//! Particle System Manager
//!
//! Coordinates particle emitters, GPU resources, and rendering.

use super::component::{ParticleEmitter, ParticleSystemHandle};
use super::compute::ParticleComputePipeline;
use super::gpu_resources::{CameraUniforms, EmitterUniforms, ParticleBufferPool};
use super::render::ParticleRenderPipeline;
use crate::core::{Entity, World};
use crate::math::Transform;
use glam::{Mat4, Vec3};

/// Central manager for all particle systems
pub struct ParticleManager {
    /// Compute pipeline
    compute: ParticleComputePipeline,
    /// Render pipeline
    render: ParticleRenderPipeline,
    /// Buffer pools per emitter
    pools: Vec<ParticleBufferPool>,
    /// Compute bind groups per pool
    compute_bind_groups: Vec<wgpu::BindGroup>,
    /// Camera bind groups per pool
    camera_bind_groups: Vec<wgpu::BindGroup>,
    /// Global time
    time: f32,
}

impl ParticleManager {
    /// Create a new particle manager
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            compute: ParticleComputePipeline::new(device),
            render: ParticleRenderPipeline::new(device, queue, surface_format),
            pools: Vec::new(),
            compute_bind_groups: Vec::new(),
            camera_bind_groups: Vec::new(),
            time: 0.0,
        }
    }

    /// Register a new emitter and allocate GPU resources
    pub fn register_emitter(
        &mut self,
        device: &wgpu::Device,
        max_particles: u32,
    ) -> ParticleSystemHandle {
        let pool = ParticleBufferPool::new(device, max_particles);
        let compute_bind_group = self.compute.create_bind_group(device, &pool);
        let camera_bind_group = self.render.create_camera_bind_group(device, &pool);

        let index = self.pools.len();
        self.pools.push(pool);
        self.compute_bind_groups.push(compute_bind_group);
        self.camera_bind_groups.push(camera_bind_group);

        ParticleSystemHandle {
            buffer_index: index,
            alive_count: 0,
            max_particles,
        }
    }

    /// Update all particle emitters
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        world: &mut World,
        camera_view_proj: Mat4,
        camera_pos: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
        delta_time: f32,
    ) {
        self.time += delta_time;

        // Update camera uniforms
        let camera_uniforms = CameraUniforms {
            view_proj: camera_view_proj.to_cols_array_2d(),
            position: [camera_pos.x, camera_pos.y, camera_pos.z, 0.0],
            right: [camera_right.x, camera_right.y, camera_right.z, 0.0],
            up: [camera_up.x, camera_up.y, camera_up.z, 0.0],
        };
        self.render.update_camera(queue, &camera_uniforms);

        // First pass: collect entity transforms (immutable borrow)
        let transforms: std::collections::HashMap<u32, Mat4> =
            if let Some(storage) = world.storage::<ParticleEmitter>() {
                storage
                    .iter()
                    .filter_map(|(entity_idx, _)| {
                        let entity = Entity::from_bits(entity_idx as u64);
                        world.get::<Transform>(entity).map(|t| {
                            (
                                entity_idx,
                                Mat4::from_scale_rotation_translation(
                                    t.scale.to_vec3(),
                                    t.rotation.quat,
                                    t.position.to_vec3(),
                                ),
                            )
                        })
                    })
                    .collect()
            } else {
                std::collections::HashMap::new()
            };

        // Second pass: update emitters (mutable borrow)
        let mut emitter_updates: Vec<EmitterUpdate> = Vec::new();

        if let Some(storage) = world.storage_mut::<ParticleEmitter>() {
            for (entity_idx, emitter) in storage.iter_mut() {
                // Register if not yet registered
                if emitter.gpu_handle.is_none() && emitter.enabled {
                    emitter.gpu_handle =
                        Some(self.register_emitter(device, emitter.config.max_particles));
                }

                if let Some(handle) = emitter.gpu_handle {
                    let transform = transforms
                        .get(&entity_idx)
                        .copied()
                        .unwrap_or(Mat4::IDENTITY);

                    let spawn_count = emitter.calculate_spawn_count(delta_time);
                    emitter.advance_seed();

                    emitter_updates.push(EmitterUpdate {
                        handle,
                        config: emitter.config.clone(),
                        transform,
                        spawn_count,
                        random_seed: emitter.random_seed,
                        enabled: emitter.enabled,
                    });
                }
            }
        }

        // Process each emitter
        for update in emitter_updates {
            if !update.enabled {
                continue;
            }

            let pool = &mut self.pools[update.handle.buffer_index];

            // Reset count
            pool.reset_count(queue);

            // Build emitter uniforms
            let uniforms = EmitterUniforms {
                transform: update.transform.to_cols_array_2d(),
                gravity_dt: [
                    update.config.physics.gravity.x,
                    update.config.physics.gravity.y,
                    update.config.physics.gravity.z,
                    delta_time,
                ],
                wind_drag: [
                    update.config.physics.wind.x,
                    update.config.physics.wind.y,
                    update.config.physics.wind.z,
                    update.config.physics.drag,
                ],
                floor_params: [
                    update.config.physics.floor_y,
                    update.config.physics.collision_bounce,
                    update.config.physics.collision_friction,
                    if update.config.physics.collision_enabled {
                        1.0
                    } else {
                        0.0
                    },
                ],
                spawn_params: [
                    update.spawn_count as f32,
                    update.config.spawn.burst_count as f32,
                    self.time,
                    update.random_seed as f32 / u32::MAX as f32,
                ],
                buffer_params: [
                    update.handle.max_particles,
                    update.handle.alive_count,
                    update.config.shape.type_id(),
                    if update.config.local_space { 1 } else { 0 },
                ],
                shape_params: update.config.shape.params(),
                start_color: [
                    update.config.appearance.start_color.x,
                    update.config.appearance.start_color.y,
                    update.config.appearance.start_color.z,
                    update.config.appearance.start_color.w,
                ],
                end_color: [
                    update.config.appearance.end_color.x,
                    update.config.appearance.end_color.y,
                    update.config.appearance.end_color.z,
                    update.config.appearance.end_color.w,
                ],
                size_lifetime: [
                    update.config.appearance.start_size,
                    update.config.appearance.end_size,
                    update.config.lifetime.start,
                    update.config.lifetime.end,
                ],
                velocity_params: [
                    update.config.spawn.initial_velocity.x,
                    update.config.spawn.initial_velocity.y,
                    update.config.spawn.initial_velocity.z,
                    update.config.spawn.speed_range.start,
                ],
                velocity_params2: [
                    update.config.spawn.velocity_randomness,
                    update.config.spawn.inherit_velocity,
                    update.config.spawn.speed_range.end,
                    update.config.appearance.rotation_speed,
                ],
                camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 0.0],
                camera_right: [camera_right.x, camera_right.y, camera_right.z, 0.0],
                camera_up: [camera_up.x, camera_up.y, camera_up.z, 0.0],
            };

            pool.update_emitter(queue, &uniforms);

            // Run compute passes
            let bind_group = &self.compute_bind_groups[update.handle.buffer_index];

            // Simulate existing particles
            self.compute
                .simulate(encoder, bind_group, update.handle.max_particles);

            // Spawn new particles
            if update.spawn_count > 0 {
                self.compute.spawn(encoder, bind_group, update.spawn_count);
            }

            // Swap buffers for next frame
            pool.swap_buffers();

            // Recreate bind groups with swapped buffers
            self.compute_bind_groups[update.handle.buffer_index] =
                self.compute.create_bind_group(device, pool);
            self.camera_bind_groups[update.handle.buffer_index] =
                self.render.create_camera_bind_group(device, pool);
        }
    }

    /// Render all particle systems
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, world: &World) {
        if let Some(storage) = world.storage::<ParticleEmitter>() {
            for (_entity_idx, emitter) in storage.iter() {
                if !emitter.enabled {
                    continue;
                }

                if let Some(handle) = &emitter.gpu_handle {
                    let camera_bind_group = &self.camera_bind_groups[handle.buffer_index];

                    self.render.render(
                        render_pass,
                        camera_bind_group,
                        None,                 // Use default texture
                        handle.max_particles, // Render all (dead ones are skipped in shader)
                        emitter.config.appearance.blend_mode,
                    );
                }
            }
        }
    }

    /// Get number of registered emitters
    pub fn emitter_count(&self) -> usize {
        self.pools.len()
    }

    /// Get total estimated alive particles
    pub fn total_particles(&self, world: &World) -> u32 {
        let mut total = 0;
        if let Some(storage) = world.storage::<ParticleEmitter>() {
            for (_entity_idx, emitter) in storage.iter() {
                if let Some(handle) = &emitter.gpu_handle {
                    total += handle.alive_count;
                }
            }
        }
        total
    }
}

/// Emitter update data
struct EmitterUpdate {
    handle: ParticleSystemHandle,
    config: super::emitter::EmitterConfig,
    transform: Mat4,
    spawn_count: u32,
    random_seed: u32,
    enabled: bool,
}

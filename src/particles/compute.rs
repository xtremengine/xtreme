//! Particle Compute Pipeline
//!
//! GPU compute shaders for particle simulation.

use super::gpu_resources::ParticleBufferPool;

/// Particle compute pipeline
pub struct ParticleComputePipeline {
    /// Simulation pipeline
    simulate_pipeline: wgpu::ComputePipeline,
    /// Spawn pipeline
    spawn_pipeline: wgpu::ComputePipeline,
    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ParticleComputePipeline {
    /// Create new compute pipelines
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/particle_compute.wgsl").into(),
            ),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Compute Bind Group Layout"),
            entries: &[
                // Input particles (read)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Output particles (write)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Alive count (atomic)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Emitter uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let simulate_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Simulate Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_simulate"),
            compilation_options: Default::default(),
            cache: None,
        });

        let spawn_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Spawn Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_spawn"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            simulate_pipeline,
            spawn_pipeline,
            bind_group_layout,
        }
    }

    /// Get bind group layout
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Create a bind group for a buffer pool
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        pool: &ParticleBufferPool,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pool.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pool.write_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: pool.count_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: pool.emitter_buffer().as_entire_binding(),
                },
            ],
        })
    }

    /// Run simulation compute pass
    pub fn simulate(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_group: &wgpu::BindGroup,
        max_particles: u32,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Particle Simulate Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.simulate_pipeline);
        compute_pass.set_bind_group(0, bind_group, &[]);

        // Workgroup size is 256
        let workgroups = max_particles.div_ceil(256);
        compute_pass.dispatch_workgroups(workgroups, 1, 1);
    }

    /// Run spawn compute pass
    pub fn spawn(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_group: &wgpu::BindGroup,
        spawn_count: u32,
    ) {
        if spawn_count == 0 {
            return;
        }

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Particle Spawn Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.spawn_pipeline);
        compute_pass.set_bind_group(0, bind_group, &[]);

        // Workgroup size is 64
        let workgroups = spawn_count.div_ceil(64);
        compute_pass.dispatch_workgroups(workgroups, 1, 1);
    }
}

use crate::{
    render::{
        mesh::{Mesh, Vertex},
        shaders::Shaders,
        texture::Texture,
    },
    utils::camera::Camera,
};
use log::{error, info};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use glam::{Mat4, Vec3};

pub use wgpu::Device;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            model: Mat4::from_translation(Vec3::new(0., 0., 0.1)),
            view: Mat4::identity(),
            projection: Mat4::identity(),
        }
    }

    pub fn update_view(&mut self, camera: &Camera) {
        self.view = camera.get_view();
        self.projection = camera.get_projection();
        // info!("[Uniforms] Updated: {:#?}", camera);
    }

    pub fn update_model(&mut self, model: Vec3) {
        self.model = Mat4::from_translation(model);
    }
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

pub enum DrawType {
    Opaque,
    Transparent,
}

pub struct Renderer {
    window: Window,

    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    frame: Option<wgpu::SwapChainFrame>,
    encoder: Option<wgpu::CommandEncoder>,

    depth: Texture,
    atlas: Texture,
    accumulator: Texture,
    revealage: Texture,

    uniforms: Uniforms,
    uniform_bind_group: wgpu::BindGroup, // Model View Projection matrix
    uniform_buffer: wgpu::Buffer,

    texture_bind_group: wgpu::BindGroup, // Texture+Sampler
    final_bind_group: wgpu::BindGroup,   // Accumulator+Revealage

    opaque_pipeline: wgpu::RenderPipeline,
    transparency_pipeline: wgpu::RenderPipeline,
    final_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Voxel game")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(&event_loop)
            .expect("Couldn't create window");

        // if let Err(e) = window.set_cursor_grab(true){
        //     error!("Couldn't grab mouse, error: {}", e);
        // }

        // window.set_cursor_visible(false);
        window.set_outer_position(winit::dpi::PhysicalPosition::new(0, 0));

        let instance = wgpu::Instance::new(wgpu::BackendBit::all());

        // TODO: Why do we need to use unsafe? What changed from the previous version of wgpu?
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Adapter request failed");

        info!("{:#?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let size = window.inner_size();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            // present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let frame = None;
        let encoder = Some(
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("renderer encoder"),
            }),
        );

        let uniforms = Uniforms::new();
        // uniforms.update_view(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform bing group"),
            layout: &uniform_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
        });

        let depth = Texture::create_depth(&device, &sc_desc, "depth texture");
        let texture_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: true },
                    count: None,
                },
            ],
        });

        let path = "./res/img/glass.png";
        let atlas = Texture::from_path(&queue, &device, path).expect("Couldn't load texture");

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bg_layout,
            label: Some("atlas tex bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });

        let path = "./res/shaders/opaque";
        let shaders = Shaders::new(&device, path).expect("Couldn't load opaque shaders");

        let opaque_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                push_constant_ranges: &[],
                bind_group_layouts: &[&texture_bg_layout, &uniform_bg_layout],
            });

        let opaque_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&opaque_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shaders.vertex,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shaders.fragment,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                ..Default::default()
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor{
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                }
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        // ***************** TRANSPARENCY PIPELINE *****************
        let accumulator = Texture::create_empty(
            &device,
            &sc_desc,
            wgpu::TextureFormat::Rgba16Float,
            "accum tex",
        );
        let revealage = Texture::create_empty(
            &device,
            &sc_desc,
            wgpu::TextureFormat::R8Unorm,
            "revealage tex",
        );

        let path = "./res/shaders/transparency";
        let shaders = Shaders::new(&device, path).expect("Couldn't load transparency shaders");

        let transparency_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                push_constant_ranges: &[],
                bind_group_layouts: &[&texture_bg_layout, &uniform_bg_layout],
            });

        let transparency_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&transparency_pipeline_layout),
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &shaders.vertex,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &shaders.fragment,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    ..Default::default()
                }),
                color_states: &[
                    wgpu::ColorStateDescriptor {
                        format: wgpu::TextureFormat::Rgba16Float,
                        color_blend: wgpu::BlendDescriptor {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha_blend: wgpu::BlendDescriptor {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                    wgpu::ColorStateDescriptor {
                        format: wgpu::TextureFormat::R8Unorm,
                        color_blend: wgpu::BlendDescriptor {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcColor,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha_blend: wgpu::BlendDescriptor {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                ],
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilStateDescriptor{
                        front: wgpu::StencilStateFaceDescriptor::IGNORE,
                        back: wgpu::StencilStateFaceDescriptor::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    }
                }),
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[Vertex::desc()],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        // ***************** FINAL PIPELINE *****************

        let final_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                    },
                    count: None,
                },
            ],
        });

        let final_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &final_bg_layout,
            label: Some("tex bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&accumulator.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&revealage.view),
                },
            ],
        });

        let path = "./res/shaders/final";
        let shaders = Shaders::new(&device, path).expect("Couldn't load final shaders");

        let final_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&final_bg_layout],
                push_constant_ranges: &[],
            });

        let final_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&final_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shaders.vertex,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shaders.fragment,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                ..Default::default()
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let device = Arc::new(device);

        Self {
            window,

            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,

            frame,
            encoder,

            depth,
            atlas,
            accumulator,
            revealage,

            uniforms,
            uniform_bind_group,
            uniform_buffer,

            texture_bind_group,
            final_bind_group,

            opaque_pipeline,
            transparency_pipeline,
            final_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.depth = Texture::create_depth(&self.device, &self.sc_desc, "depth texture");
        self.accumulator = Texture::create_empty(
            &self.device,
            &self.sc_desc,
            wgpu::TextureFormat::Rgba16Float,
            "accum tex",
        );
        self.revealage = Texture::create_empty(
            &self.device,
            &self.sc_desc,
            wgpu::TextureFormat::R8Unorm,
            "revealage tex",
        );
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn start_frame(&mut self) {
        self.frame = Some(
            self.swap_chain
                .get_current_frame()
                .expect("couldn't get swap chain output"),
        ); //acquire new frame
        self.encoder = Some(
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("renderer encoder"),
                }),
        );
    }
    pub fn end_frame(&mut self) {
        if let Some(encoder) = self.encoder.take() {
            self.queue.submit(Some(encoder.finish()));
        } else {
            error!("[Frame End] Failed to acquire encoder!");
        }

        drop(self.frame.take()); // drops old frame after submitting commands
    }

    pub fn clear(&mut self) {
        if let Some(encoder) = self.encoder.as_mut() {
            if let Some(frame) = self.frame.as_mut() {
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.output.view,
                            resolve_target: None,
                            ops: wgpu::Operations{
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        b: 0.3,
                                        a: 1.0,
                                    },
                                ),
                                store: true
                            }
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.accumulator.view,
                            resolve_target: None,
                            ops: wgpu::Operations{
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.,
                                        g: 0.,
                                        b: 0.,
                                        a: 0.,
                                    },
                                ),
                                store: true
                            }
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.revealage.view,
                            resolve_target: None,
                            ops: wgpu::Operations{
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 1.,
                                        g: 0.,
                                        b: 0.,
                                        a: 0.,
                                    },
                                ),
                                store: true
                            }
                        },
                    ],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &self.depth.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0),
                                store: true,
                            }),
                        },
                    ),
                });
            } else {
                error!("[Clear Pass] Failed to acquire frame!");
            }
        } else {
            error!("[Clear Pass] Failed to acquire encoder!");
        }
    }

    pub fn draw(&mut self, draw_type: DrawType, position: &Vec3, mesh: &Mesh) {
        match draw_type {
            DrawType::Opaque => self.draw_opaque(position, mesh),
            DrawType::Transparent => self.draw_transparent(position, mesh),
        }
    }

    fn draw_opaque(&mut self, position: &Vec3, mesh: &Mesh) {
        if let Some(encoder) = self.encoder.as_mut() {
            if let Some(frame) = self.frame.as_mut() {
                self.uniforms.update_model(*position);
                let staging_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Staging buffer"),
                    contents: bytemuck::cast_slice(&[self.uniforms]),
                    usage: wgpu::BufferUsage::COPY_SRC,
                });

                encoder.copy_buffer_to_buffer(
                    &staging_buffer,
                    0,
                    &self.uniform_buffer,
                    0,
                    std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
                );

                let mut opaque_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations{
                            load: wgpu::LoadOp::Load,
                            store: true
                        }
                    }],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &self.depth.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }),
                            stencil_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }),
                        },
                    ),
                });

                opaque_pass.set_pipeline(&self.opaque_pipeline);

                opaque_pass.set_bind_group(0, &self.texture_bind_group, &[]);
                opaque_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
                opaque_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                opaque_pass.set_index_buffer(mesh.index_buffer.slice(..));

                opaque_pass.draw_indexed(0..mesh.size, 0, 0..1);
            } else {
                error!("[Opaque Draw] Failed to acquire frame!");
            }
        } else {
            error!("[Opaque Draw] Failed to acquire encoder!");
        }
    }

    fn draw_transparent(&mut self, position: &Vec3, mesh: &Mesh) {
        if let Some(encoder) = self.encoder.as_mut() {
            self.uniforms.update_model(*position);
            let staging_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Staging buffer"),
                contents: bytemuck::cast_slice(&[self.uniforms]),
                usage: wgpu::BufferUsage::COPY_SRC,
            });

            encoder.copy_buffer_to_buffer(
                &staging_buffer,
                0,
                &self.uniform_buffer,
                0,
                std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            );

            let mut transparency_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.accumulator.view,
                        resolve_target: None,
                        ops: wgpu::Operations{
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }
                    },
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.revealage.view,
                        resolve_target: None,
                        ops: wgpu::Operations{
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }
                    },
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth.view,
                    depth_ops: Some(wgpu::Operations{
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations{
                        load: wgpu::LoadOp::Load,
                        store: true,
                    })
                }),
            });

            transparency_pass.set_pipeline(&self.transparency_pipeline);

            transparency_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            transparency_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            transparency_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            transparency_pass.set_index_buffer(mesh.index_buffer.slice(..));

            transparency_pass.draw_indexed(0..mesh.size, 0, 0..1);
        } else {
            error!("[Transparent Draw] Failed to acquire encoder!");
        }
    }

    pub fn final_pass(&mut self) {
        if let Some(encoder) = self.encoder.as_mut() {
            if let Some(frame) = self.frame.as_mut() {
                let mut screen_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations{
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }
                    }],
                    depth_stencil_attachment: None,
                });

                screen_pass.set_pipeline(&self.final_pipeline);
                screen_pass.set_bind_group(0, &self.final_bind_group, &[]);

                screen_pass.draw(0..6, 0..1);
            } else {
                error!("[Final Pass] Failed to acquire frame!");
            }
        } else {
            error!("[Final Pass] Failed to acquire encoder!");
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn arc_device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut wgpu::Device {
        Arc::get_mut(&mut self.device).expect("Failed to get mutable wgpu Device")
    }

    pub fn uniforms(&mut self) -> &mut Uniforms {
        &mut self.uniforms
    }
}

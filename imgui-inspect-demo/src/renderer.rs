use rafx::api::*;
use rafx::nodes::{RenderRegistry, RenderRegistryBuilder};

/// Vulkan renderer that creates and manages the vulkan instance, device, swapchain, and
/// render passes.
pub struct Renderer {
    // Ordered in drop order
    debug_pipeline: RafxPipeline,
    imgui_pipeline: RafxPipeline,
    command_buffers: Vec<RafxCommandBuffer>,
    command_pools: Vec<RafxCommandPool>,
    graphics_queue: RafxQueue,
    swapchain_helper: RafxSwapchainHelper,
    api: RafxApi,
}

impl Renderer {
    /// Create the renderer
    pub fn new(
        window: &winit::window::Window,
    ) -> RafxResult<Renderer> {
        let window_size = window.inner_size();
        let window_size = RafxExtents2D {
            width: window_size.width,
            height: window_size.height,
        };

        let mut api = RafxApi::new(window, &Default::default())?;
        let device_context = api.device_context();

        let render_registry = RenderRegistryBuilder::default().build();
        let resource_manager = rafx::framework::ResourceManager::new(&device_context, &render_registry);

        let swapchain = device_context.create_swapchain(
            window,
            &RafxSwapchainDef {
                width: window_size.width,
                height: window_size.height,
                enable_vsync: true,
            },
        )?;
        let mut swapchain_helper = RafxSwapchainHelper::new(&device_context, swapchain, None)?;
        let graphics_queue = device_context.create_queue(RafxQueueType::Graphics)?;
        let mut command_pools = Vec::with_capacity(swapchain_helper.image_count());
        let mut command_buffers = Vec::with_capacity(swapchain_helper.image_count());

        for _ in 0..swapchain_helper.image_count() {
            let mut command_pool =
                graphics_queue.create_command_pool(&RafxCommandPoolDef { transient: true })?;

            let command_buffer = command_pool.create_command_buffer(&RafxCommandBufferDef {
                is_secondary: false,
            })?;

            command_pools.push(command_pool);
            command_buffers.push(command_buffer);
        }

        let color_format = swapchain_helper.format();

        let debug_pipeline = Self::load_debug_pipeline(&device_context, color_format)?;
        let imgui_pipeline = Self::load_imgui_pipeline(&device_context, color_format)?;







        //let mut skia_context = ManuallyDrop::new(VkSkiaContext::new(&instance, &device));
        // let swapchain = ManuallyDrop::new(VkSwapchain::new(
        //     &instance,
        //     &device,
        //     window,
        //     None,
        //     &present_mode_priority,
        // )?);
        // let skia_renderpass = ManuallyDrop::new(VkSkiaRenderPass::new(
        //     &device,
        //     &swapchain,
        //     &mut skia_context,
        // )?);
        //
        // for plugin in &mut plugins {
        //     plugin.swapchain_created(&device, &swapchain)?;
        // }

        Ok(Renderer {
            api,
            swapchain_helper,
            graphics_queue,
            command_pools,
            command_buffers,
            debug_pipeline,
            imgui_pipeline
        })
    }

    // pub fn skia_context(&self) -> &skia_safe::gpu::Context {
    //     &self.skia_context.context
    // }

    /// Call to render a frame. This can block for certain presentation modes. This will rebuild
    /// the swapchain if necessary.
    pub fn draw<F: FnOnce(&RafxCommandBuffer)>(
        &mut self,
        window: &winit::window::Window,
        f: F,
    ) -> RafxResult<()> {
        let window_size = window.inner_size();
        let window_size = RafxExtents2D {
            width: window_size.width,
            height: window_size.height,
        };

        let frame = self.swapchain_helper.acquire_next_image(
            window_size.width,
            window_size.height,
            None,
        )?;

        self.command_pools[frame.rotating_frame_index()].reset_command_pool()?;
        let command_buffer = &self.command_buffers[frame.rotating_frame_index()];
        command_buffer.begin()?;
        command_buffer.end()?;

        frame.present(&self.graphics_queue, &[&command_buffer])?;

        Ok(())
    }

    fn load_debug_pipeline(device_context: &RafxDeviceContext, color_format: RafxFormat) -> RafxResult<RafxPipeline> {
        let mut vert_package = RafxShaderPackage {
            metal: None,
            vk: Some(RafxShaderPackageVulkan::SpvBytes(
                include_bytes!("../shaders/out/debug.vert.spv").to_vec(),
            )),
        };

        let mut frag_package = RafxShaderPackage {
            metal: None,
            vk: Some(RafxShaderPackageVulkan::SpvBytes(
                include_bytes!("../shaders/out/debug.frag.spv").to_vec(),
            )),
        };

        let vert_shader_module = device_context.create_shader_module(vert_package.module_def())?;
        let frag_shader_module = device_context.create_shader_module(frag_package.module_def())?;

        let vert_shader_stage_def = RafxShaderStageDef {
            shader_module: vert_shader_module,
            reflection: RafxShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: RafxShaderStageFlags::VERTEX,
                compute_threads_per_group: None,
                resources: vec![],
            },
        };

        let frag_shader_stage_def = RafxShaderStageDef {
            shader_module: frag_shader_module,
            reflection: RafxShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: RafxShaderStageFlags::FRAGMENT,
                compute_threads_per_group: None,
                resources: vec![],
            },
        };

        let shader =
            device_context.create_shader(vec![vert_shader_stage_def, frag_shader_stage_def])?;

        let root_signature = device_context.create_root_signature(&RafxRootSignatureDef {
            shaders: &[shader.clone()],
            immutable_samplers: &[],
        })?;

        let vertex_layout = RafxVertexLayout {
            attributes: vec![
                RafxVertexLayoutAttribute {
                    format: RafxFormat::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 0,
                    offset: 0,
                },
                RafxVertexLayoutAttribute {
                    format: RafxFormat::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    offset: 8,
                },
            ],
            buffers: vec![RafxVertexLayoutBuffer {
                stride: 16,
                rate: RafxVertexAttributeRate::Vertex,
            }],
        };

        let pipeline = device_context.create_graphics_pipeline(&RafxGraphicsPipelineDef {
            shader: &shader,
            root_signature: &root_signature,
            vertex_layout: &vertex_layout,
            blend_state: &Default::default(),
            depth_state: &Default::default(),
            rasterizer_state: &Default::default(),
            color_formats: &[color_format],
            sample_count: RafxSampleCount::SampleCount1,
            depth_stencil_format: None,
            primitive_topology: RafxPrimitiveTopology::TriangleStrip,
        })?;

        Ok(pipeline)
    }

    fn load_imgui_pipeline(device_context: &RafxDeviceContext, color_format: RafxFormat) -> RafxResult<RafxPipeline> {
        let mut vert_package = RafxShaderPackage {
            metal: None,
            vk: Some(RafxShaderPackageVulkan::SpvBytes(
                include_bytes!("../shaders/out/imgui.vert.spv").to_vec(),
            )),
        };

        let mut frag_package = RafxShaderPackage {
            metal: None,
            vk: Some(RafxShaderPackageVulkan::SpvBytes(
                include_bytes!("../shaders/out/imgui.frag.spv").to_vec(),
            )),
        };

        let vert_shader_module = device_context.create_shader_module(vert_package.module_def())?;
        let frag_shader_module = device_context.create_shader_module(frag_package.module_def())?;

        let shader_resources = vec![
            RafxShaderResource {
                resource_type: RafxResourceType::SAMPLER,
                name: Some("str".to_string()),
                set_index: 0,
                binding: 0,
                ..Default::default()
            },
            RafxShaderResource {
                resource_type: RafxResourceType::TEXTURE,
                name: Some("tex".to_string()),
                set_index: 0,
                binding: 1,
                ..Default::default()
            },
            RafxShaderResource {
                resource_type: RafxResourceType::TEXTURE,
                name: Some("uniform_buffer".to_string()),
                set_index: 0,
                binding: 1,
                ..Default::default()
            },
        ];

        let vert_shader_stage_def = RafxShaderStageDef {
            shader_module: vert_shader_module,
            reflection: RafxShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: RafxShaderStageFlags::VERTEX,
                compute_threads_per_group: None,
                resources: shader_resources.clone(),
            },
        };

        let frag_shader_stage_def = RafxShaderStageDef {
            shader_module: frag_shader_module,
            reflection: RafxShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: RafxShaderStageFlags::FRAGMENT,
                compute_threads_per_group: None,
                resources: shader_resources,
            },
        };

        let shader =
            device_context.create_shader(vec![vert_shader_stage_def, frag_shader_stage_def])?;

        // let root_signature = device_context.create_root_signature(&RafxRootSignatureDef {
        //     shaders: &[shader.clone()],
        //     immutable_samplers: &[],
        // })?;

        let sampler = device_context.create_sampler(&RafxSamplerDef {
            mag_filter: RafxFilterType::Linear,
            min_filter: RafxFilterType::Linear,
            address_mode_u: RafxAddressMode::Mirror,
            address_mode_v: RafxAddressMode::Mirror,
            address_mode_w: RafxAddressMode::Mirror,
            compare_op: RafxCompareOp::Never,
            mip_map_mode: RafxMipMapMode::Linear,
            max_anisotropy: 1.0,
            mip_lod_bias: 0.0,
        })?;

        let root_signature = device_context.create_root_signature(&RafxRootSignatureDef {
            shaders: &[shader.clone()],
            immutable_samplers: &[RafxImmutableSamplers {
                key: RafxImmutableSamplerKey::from_binding(0, 0),
                samplers: &[sampler],
            }],
        })?;

        let vertex_layout = RafxVertexLayout {
            attributes: vec![
                RafxVertexLayoutAttribute {
                    format: RafxFormat::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 0,
                    offset: 0,
                },
                RafxVertexLayoutAttribute {
                    format: RafxFormat::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    offset: 8,
                },
            ],
            buffers: vec![RafxVertexLayoutBuffer {
                stride: 16,
                rate: RafxVertexAttributeRate::Vertex,
            }],
        };

        let pipeline = device_context.create_graphics_pipeline(&RafxGraphicsPipelineDef {
            shader: &shader,
            root_signature: &root_signature,
            vertex_layout: &vertex_layout,
            blend_state: &Default::default(),
            depth_state: &Default::default(),
            rasterizer_state: &Default::default(),
            color_formats: &[color_format],
            sample_count: RafxSampleCount::SampleCount1,
            depth_stencil_format: None,
            primitive_topology: RafxPrimitiveTopology::TriangleStrip,
        })?;

        Ok(pipeline)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::debug!("destroying Renderer");
        self.graphics_queue.wait_for_queue_idle();
        log::debug!("destroyed Renderer");
    }
}

use rafx::api::*;
use rafx::nodes::*;
use rafx::framework::*;
use std::sync::Arc;

rafx::nodes::declare_render_phase!(
    OpaqueRenderPhase,
    OPAQUE_RENDER_PHASE_INDEX,
    opaque_render_phase_sort_submit_nodes
);

fn opaque_render_phase_sort_submit_nodes(mut submit_nodes: Vec<SubmitNode>) -> Vec<SubmitNode> {
    submit_nodes.sort_unstable_by(|a, b| a.feature_index().cmp(&b.feature_index()));
    submit_nodes
}

lazy_static::lazy_static! {
    pub static ref IMGUI_VERTEX_LAYOUT : VertexDataSetLayout = {
        use rafx::api::RafxFormat;

        let vertex = imgui::DrawVert {
            pos: Default::default(),
            col: Default::default(),
            uv: Default::default()
        };

        VertexDataLayout::build_vertex_layout(&vertex, |builder, vertex| {
            builder.add_member(&vertex.pos, "POSITION", RafxFormat::R32G32_SFLOAT);
            builder.add_member(&vertex.uv, "TEXCOORD", RafxFormat::R32G32_SFLOAT);
            builder.add_member(&vertex.col, "COLOR", RafxFormat::R8G8B8A8_UNORM);
        }).into_set(RafxPrimitiveTopology::TriangleList)
    };
}

#[derive(Clone, Copy)]
struct DebugVertex {
    position: [f32; 2],
    color: [f32; 4]
}

lazy_static::lazy_static! {
    pub static ref DEBUG_VERTEX_LAYOUT : VertexDataSetLayout = {
        use rafx::api::RafxFormat;

        let vertex = DebugVertex {
            position: Default::default(),
            color: Default::default(),
        };

        VertexDataLayout::build_vertex_layout(&vertex, |builder, vertex| {
            builder.add_member(&vertex.position, "POSITION", RafxFormat::R32G32_SFLOAT);
            builder.add_member(&vertex.color, "COLOR", RafxFormat::R32G32B32A32_SFLOAT);
        }).into_set(RafxPrimitiveTopology::TriangleList)
    };
}

/// Vulkan renderer that creates and manages the vulkan instance, device, swapchain, and
/// render passes.
pub struct Renderer {
    // Ordered in drop order
    debug_pass: MaterialPass,
    imgui_pass: MaterialPass,
    graphics_queue: RafxQueue,
    swapchain_helper: RafxSwapchainHelper,
    resource_manager: ResourceManager,
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

        let render_registry = RenderRegistryBuilder::default()
            .register_render_phase::<OpaqueRenderPhase>("opaque")
            .build();
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

        let resource_context = resource_manager.resource_context();

        let debug_pass = Self::load_material_pass(
            &resource_context,
            include_bytes!("../shaders/out/debug.vert.cookedshaderpackage"),
            include_bytes!("../shaders/out/debug.frag.cookedshaderpackage"),
            FixedFunctionState {
                rasterizer_state: Default::default(),
                depth_state: Default::default(),
                blend_state: Default::default()
            }
        )?;

        let imgui_pass = Self::load_material_pass(
            &resource_context,
            include_bytes!("../shaders/out/imgui.vert.cookedshaderpackage"),
            include_bytes!("../shaders/out/imgui.frag.cookedshaderpackage"),
            FixedFunctionState {
                rasterizer_state: Default::default(),
                depth_state: Default::default(),
                blend_state: Default::default()
            }
        )?;

        Ok(Renderer {
            api,
            swapchain_helper,
            graphics_queue,
            debug_pass,
            imgui_pass,
            resource_manager,
        })
    }

    /// Call to render a frame. This can block for certain presentation modes. This will rebuild
    /// the swapchain if necessary.
    pub fn draw(
        &mut self,
        window: &winit::window::Window,
        imgui_draw_data: &imgui::DrawData,
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

        self.resource_manager.on_frame_complete();

        let mut command_pool = self.resource_manager.dyn_command_pool_allocator().allocate_dyn_pool(
            &self.graphics_queue,
            &RafxCommandPoolDef {
                transient: false
            },
            0
        )?;

        let command_buffer = command_pool.allocate_dyn_command_buffer(&RafxCommandBufferDef {
            is_secondary: false
        })?;

        command_buffer.begin()?;

        command_buffer.cmd_resource_barrier(
            &[],
            &[
                RafxTextureBarrier {
                    texture: frame.swapchain_texture(),
                    array_slice: None,
                    mip_slice: None,
                    src_state: RafxResourceState::PRESENT,
                    dst_state: RafxResourceState::RENDER_TARGET,
                    queue_transition: RafxBarrierQueueTransition::None
                }
            ]
        );

        command_buffer.cmd_begin_render_pass(
            &[RafxColorRenderTargetBinding {
                texture: frame.swapchain_texture(),
                load_op: RafxLoadOp::Clear,
                store_op: RafxStoreOp::Store,
                clear_value: RafxColorClearValue([0.0, 0.0, 0.0, 0.0]),
                mip_slice: Default::default(),
                array_slice: Default::default(),
                resolve_target: Default::default(),
                resolve_store_op: Default::default(),
                resolve_mip_slice: Default::default(),
                resolve_array_slice: Default::default(),
            }],
            None
        )?;

        self.draw_debug(&command_buffer);
        self.draw_imgui(&command_buffer, imgui_draw_data);

        command_buffer.cmd_end_render_pass()?;

        command_buffer.cmd_resource_barrier(
            &[],
            &[
                RafxTextureBarrier {
                    texture: frame.swapchain_texture(),
                    array_slice: None,
                    mip_slice: None,
                    src_state: RafxResourceState::RENDER_TARGET,
                    dst_state: RafxResourceState::PRESENT,
                    queue_transition: RafxBarrierQueueTransition::None
                }
            ]
        );

        command_buffer.end()?;

        frame.present(&self.graphics_queue, &[&command_buffer])?;

        Ok(())
    }

    fn draw_debug(&self, command_buffer: &RafxCommandBuffer) -> RafxResult<()> {
        let debug_pipeline = self.resource_manager.graphics_pipeline_cache().get_or_create_graphics_pipeline(
            OpaqueRenderPhase::render_phase_index(),
            &self.debug_pass.material_pass_resource,
            &GraphicsPipelineRenderTargetMeta::new(
                vec![self.swapchain_helper.format()],
                None,
                RafxSampleCount::SampleCount1
            ),
            &DEBUG_VERTEX_LAYOUT
        )?;
        command_buffer.cmd_bind_pipeline(&*debug_pipeline.get_raw().pipeline)?;

        Ok(())
    }

    fn draw_imgui(
        &self,
        command_buffer: &RafxCommandBuffer,
        imgui_draw_data: &imgui::DrawData,
    ) -> RafxResult<()> {
        let device_context = self.dev
        let draw_list_count = imgui_draw_data.draw_lists_count();

        let mut vertex_buffers = Vec::with_capacity(draw_list_count);
        let mut index_buffers = Vec::with_capacity(draw_list_count);
        if let Some(draw_data) = Some(imgui_draw_data) {
            for draw_list in draw_data.draw_lists() {
                let vertex_buffer_size = draw_list.vertex_buffer().len() as u64
                    * std::mem::size_of::<imgui::DrawVert>() as u64;

                let vertex_buffer = device_context
                    .create_buffer(&RafxBufferDef {
                        size: vertex_buffer_size,
                        memory_usage: RafxMemoryUsage::CpuToGpu,
                        resource_type: RafxResourceType::VERTEX_BUFFER,
                        ..Default::default()
                    })
                    .unwrap();

                vertex_buffer
                    .copy_to_host_visible_buffer(draw_list.vertex_buffer())
                    .unwrap();
                let vertex_buffer = dyn_resource_allocator.insert_buffer(vertex_buffer);

                let index_buffer_size = draw_list.index_buffer().len() as u64
                    * std::mem::size_of::<imgui::DrawIdx>() as u64;

                let index_buffer = prepare_context
                    .device_context
                    .create_buffer(&RafxBufferDef {
                        size: index_buffer_size,
                        memory_usage: RafxMemoryUsage::CpuToGpu,
                        resource_type: RafxResourceType::INDEX_BUFFER,
                        ..Default::default()
                    })
                    .unwrap();

                index_buffer
                    .copy_to_host_visible_buffer(draw_list.index_buffer())
                    .unwrap();
                let index_buffer = dyn_resource_allocator.insert_buffer(index_buffer);

                vertex_buffers.push(vertex_buffer);
                index_buffers.push(index_buffer);
            }
        }

        let imgui_pipeline = self.resource_manager.graphics_pipeline_cache().get_or_create_graphics_pipeline(
            OpaqueRenderPhase::render_phase_index(),
            &self.imgui_pass.material_pass_resource,
            &GraphicsPipelineRenderTargetMeta::new(
                vec![self.swapchain_helper.format()],
                None,
                RafxSampleCount::SampleCount1
            ),
            &IMGUI_VERTEX_LAYOUT
        )?;
        command_buffer.cmd_bind_pipeline(&*imgui_pipeline.get_raw().pipeline)?;

        Ok(())
    }

    fn load_material_pass(
        resource_context: &ResourceContext,
        cooked_vertex_shader_bytes: &[u8],
        cooked_fragment_shader_bytes: &[u8],
        fixed_function_state: FixedFunctionState
    ) -> RafxResult<MaterialPass> {
        let cooked_vertex_shader_stage = bincode::deserialize::<CookedShaderPackage>(cooked_vertex_shader_bytes)
            .map_err(|x| format!("Failed to deserialize cooked shader: {:?}", x))?;

        let vertex_shader_module = resource_context
            .resources()
            .get_or_create_shader_module_from_cooked_package(&cooked_vertex_shader_stage)?;
        let vertex_entry_point = cooked_vertex_shader_stage
            .find_entry_point("main")
            .unwrap()
            .clone();

        // Create the fragment shader module and find the entry point
        let cooked_fragment_shader_stage = bincode::deserialize::<CookedShaderPackage>(cooked_fragment_shader_bytes)
            .map_err(|x| format!("Failed to deserialize cooked shader: {:?}", x))?;
        let fragment_shader_module = resource_context
            .resources()
            .get_or_create_shader_module_from_cooked_package(&cooked_fragment_shader_stage)?;
        let fragment_entry_point = cooked_fragment_shader_stage
            .find_entry_point("main")
            .unwrap()
            .clone();

        //
        // Now set up the fixed function and vertex input state. LOTS of things can be configured
        // here, but aside from the vertex layout most of it can be left as default.
        //
        let fixed_function_state = Arc::new(fixed_function_state);

        // Creating a material automatically registers the necessary resources in the resource
        // manager and caches references to them. (This is almost the same as loading a material
        // asset, although a material asset can have multiple passes).
        let material_pass = MaterialPass::new(
            &resource_context,
            fixed_function_state,
            vec![vertex_shader_module, fragment_shader_module],
            &[&vertex_entry_point, &fragment_entry_point],
        )?;

        Ok(material_pass)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::debug!("destroying Renderer");
        self.graphics_queue.wait_for_queue_idle();
        log::debug!("destroyed Renderer");
    }
}

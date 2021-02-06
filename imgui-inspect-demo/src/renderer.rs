use rafx::api::*;
use rafx::nodes::*;
use rafx::framework::*;
use std::sync::Arc;
use crate::ExampleInspectTarget;

rafx::nodes::declare_render_phase!(
    OpaqueRenderPhase,
    OPAQUE_RENDER_PHASE_INDEX,
    opaque_render_phase_sort_submit_nodes
);

fn opaque_render_phase_sort_submit_nodes(submit_nodes: Vec<SubmitNode>) -> Vec<SubmitNode> {
    // No sort needed
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

#[derive(Debug, Clone, Copy)]
struct DebugVertex {
    position: [f32; 2],
    color: [f32; 4],
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
    font_atlas_texture: ResourceArc<ImageViewResource>,
    debug_pass: MaterialPass,
    imgui_pass: MaterialPass,
    graphics_queue: RafxQueue,
    swapchain_helper: RafxSwapchainHelper,
    resource_manager: ResourceManager,
    #[allow(dead_code)]
    api: RafxApi,
}

impl Renderer {
    /// Create the renderer
    pub fn new(
        window: &winit::window::Window,
        font_atlas_texture: &imgui::FontAtlasTexture,
    ) -> RafxResult<Renderer> {
        let window_size = window.inner_size();
        let window_size = RafxExtents2D {
            width: window_size.width,
            height: window_size.height,
        };

        let api = RafxApi::new(window, &Default::default())?;
        let device_context = api.device_context();

        let render_registry = RenderRegistryBuilder::default()
            .register_render_phase::<OpaqueRenderPhase>("opaque")
            .build();
        let resource_manager =
            rafx::framework::ResourceManager::new(&device_context, &render_registry);

        let swapchain = device_context.create_swapchain(
            window,
            &RafxSwapchainDef {
                width: window_size.width,
                height: window_size.height,
                enable_vsync: true,
            },
        )?;

        let swapchain_helper = RafxSwapchainHelper::new(&device_context, swapchain, None)?;
        let graphics_queue = device_context.create_queue(RafxQueueType::Graphics)?;

        let resource_context = resource_manager.resource_context();

        let debug_pass = Self::load_material_pass(
            &resource_context,
            include_bytes!("../shaders/out/debug.vert.cookedshaderpackage"),
            include_bytes!("../shaders/out/debug.frag.cookedshaderpackage"),
            FixedFunctionState {
                rasterizer_state: Default::default(),
                depth_state: Default::default(),
                blend_state: Default::default(),
            },
        )?;

        let imgui_pass = Self::load_material_pass(
            &resource_context,
            include_bytes!("../shaders/out/imgui.vert.cookedshaderpackage"),
            include_bytes!("../shaders/out/imgui.frag.cookedshaderpackage"),
            FixedFunctionState {
                rasterizer_state: Default::default(),
                depth_state: Default::default(),
                blend_state: RafxBlendState::default_alpha_enabled(),
            },
        )?;

        let font_atlas_texture =
            Self::upload_font_atlas(&resource_manager, &graphics_queue, font_atlas_texture)?;

        Ok(Renderer {
            api,
            swapchain_helper,
            graphics_queue,
            debug_pass,
            imgui_pass,
            resource_manager,
            font_atlas_texture,
        })
    }

    pub fn upload_font_atlas(
        resource_manager: &ResourceManager,
        queue: &RafxQueue,
        font_atlas_texture: &imgui::FontAtlasTexture,
    ) -> RafxResult<ResourceArc<ImageViewResource>> {
        let mut command_pool = resource_manager
            .dyn_command_pool_allocator()
            .allocate_dyn_pool(queue, &RafxCommandPoolDef { transient: true }, 0)?;

        let command_buffer = command_pool.allocate_dyn_command_buffer(&RafxCommandBufferDef {
            is_secondary: false,
        })?;

        let buffer = resource_manager
            .device_context()
            .create_buffer(&RafxBufferDef {
                size: font_atlas_texture.data.len() as u64,
                alignment: 0,
                memory_usage: RafxMemoryUsage::CpuToGpu,
                queue_type: RafxQueueType::Graphics,
                resource_type: RafxResourceType::BUFFER,
                elements: Default::default(),
                format: RafxFormat::UNDEFINED,
                always_mapped: true,
            })?;

        let texture = resource_manager
            .device_context()
            .create_texture(&RafxTextureDef {
                extents: RafxExtents3D {
                    width: font_atlas_texture.width,
                    height: font_atlas_texture.height,
                    depth: 1,
                },
                format: RafxFormat::R8G8B8A8_UNORM,
                ..Default::default()
            })?;

        buffer.copy_to_host_visible_buffer(font_atlas_texture.data)?;

        command_buffer.begin()?;
        command_buffer.cmd_resource_barrier(
            &[],
            &[RafxTextureBarrier::state_transition(
                &texture,
                RafxResourceState::UNDEFINED,
                RafxResourceState::COPY_DST,
            )],
        )?;
        command_buffer.cmd_copy_buffer_to_texture(
            &buffer,
            &texture,
            &RafxCmdCopyBufferToTextureParams::default(),
        )?;

        command_buffer.cmd_resource_barrier(
            &[],
            &[RafxTextureBarrier::state_transition(
                &texture,
                RafxResourceState::COPY_DST,
                RafxResourceState::SHADER_RESOURCE,
            )],
        )?;
        command_buffer.end()?;
        queue.submit(&[&command_buffer], &[], &[], None)?;
        queue.wait_for_queue_idle()?;

        let image = resource_manager.resources().insert_image(texture);
        resource_manager
            .resources()
            .get_or_create_image_view(&image, None)
    }

    /// Call to render a frame. This can block for certain presentation modes. This will rebuild
    /// the swapchain if necessary.
    pub fn draw(
        &mut self,
        window: &winit::window::Window,
        imgui_draw_data: Option<&imgui::DrawData>,
        example_inspect_target: &ExampleInspectTarget,
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

        self.resource_manager.on_frame_complete()?;

        let mut command_pool = self
            .resource_manager
            .dyn_command_pool_allocator()
            .allocate_dyn_pool(
                &self.graphics_queue,
                &RafxCommandPoolDef { transient: false },
                0,
            )?;

        let command_buffer = command_pool.allocate_dyn_command_buffer(&RafxCommandBufferDef {
            is_secondary: false,
        })?;

        command_buffer.begin()?;

        command_buffer.cmd_resource_barrier(
            &[],
            &[RafxTextureBarrier {
                texture: frame.swapchain_texture(),
                array_slice: None,
                mip_slice: None,
                src_state: RafxResourceState::PRESENT,
                dst_state: RafxResourceState::RENDER_TARGET,
                queue_transition: RafxBarrierQueueTransition::None,
            }],
        )?;

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
            None,
        )?;

        self.draw_debug(&command_buffer, example_inspect_target)?;
        self.draw_imgui(&command_buffer, imgui_draw_data)?;

        command_buffer.cmd_end_render_pass()?;

        command_buffer.cmd_resource_barrier(
            &[],
            &[RafxTextureBarrier {
                texture: frame.swapchain_texture(),
                array_slice: None,
                mip_slice: None,
                src_state: RafxResourceState::RENDER_TARGET,
                dst_state: RafxResourceState::PRESENT,
                queue_transition: RafxBarrierQueueTransition::None,
            }],
        )?;

        command_buffer.end()?;

        frame.present(&self.graphics_queue, &[&command_buffer])?;

        Ok(())
    }

    fn draw_debug(
        &self,
        command_buffer: &RafxCommandBuffer,
        example_inspect_target: &ExampleInspectTarget,
    ) -> RafxResult<()> {
        let primitive_count = 50;
        let mut vertices = Vec::with_capacity(primitive_count * 3);

        let x_position = example_inspect_target.x_position;
        let y_position = example_inspect_target.y_position;
        let radius = example_inspect_target.radius;

        let color = [
            example_inspect_target.color.0.r,
            example_inspect_target.color.0.g,
            example_inspect_target.color.0.b,
            example_inspect_target.color.0.a,
        ];

        for i in 0..primitive_count {
            let two_pi = std::f32::consts::PI * 2.0;
            let p1 = two_pi * (i as f32 / primitive_count as f32);
            let p2 = two_pi * ((i as f32 + 1.0) / primitive_count as f32);

            vertices.push(DebugVertex {
                position: [
                    x_position + (p1.cos() * radius),
                    y_position + (p1.sin() * radius),
                ],
                color,
            });

            vertices.push(DebugVertex {
                position: [
                    x_position + (p2.cos() * radius),
                    y_position + (p2.sin() * radius),
                ],
                color,
            });

            vertices.push(DebugVertex {
                position: [x_position, y_position],
                color,
            });
        }

        let vertex_buffer = self.resource_manager.device_context().create_buffer(
            &RafxBufferDef::for_staging_vertex_buffer_data(&vertices[..]),
        )?;
        vertex_buffer.copy_to_host_visible_buffer(&vertices[..])?;
        let vertex_buffer = self
            .resource_manager
            .create_dyn_resource_allocator_set()
            .insert_buffer(vertex_buffer);

        let mut descriptor_set_allocator = self.resource_manager.create_descriptor_set_allocator();
        let mut descriptor_set = descriptor_set_allocator.create_dyn_descriptor_set_uninitialized(
            &self
                .debug_pass
                .material_pass_resource
                .get_raw()
                .descriptor_set_layouts[0],
        )?;

        let dpi_scaling = 1.0;
        let top = 0.0;
        let bottom = self.swapchain_helper.swapchain_def().height as f32 / dpi_scaling;
        let view_proj = glam::Mat4::orthographic_rh(
            0.0,
            self.swapchain_helper.swapchain_def().width as f32 / dpi_scaling,
            bottom,
            top,
            -100.0,
            100.0,
        );

        descriptor_set.set_buffer_data(0, &view_proj);

        descriptor_set.flush(&mut descriptor_set_allocator)?;
        descriptor_set_allocator.flush_changes()?;

        descriptor_set.bind(command_buffer)?;

        let debug_pipeline = self
            .resource_manager
            .graphics_pipeline_cache()
            .get_or_create_graphics_pipeline(
                OpaqueRenderPhase::render_phase_index(),
                &self.debug_pass.material_pass_resource,
                &GraphicsPipelineRenderTargetMeta::new(
                    vec![self.swapchain_helper.format()],
                    None,
                    RafxSampleCount::SampleCount1,
                ),
                &DEBUG_VERTEX_LAYOUT,
            )?;
        command_buffer.cmd_bind_pipeline(&*debug_pipeline.get_raw().pipeline)?;

        command_buffer.cmd_bind_vertex_buffers(
            0,
            &[RafxVertexBufferBinding {
                buffer: &*vertex_buffer.get_raw().buffer,
                offset: 0,
            }],
        )?;

        command_buffer.cmd_draw(primitive_count as u32 * 3, 0)?;

        Ok(())
    }

    fn draw_imgui(
        &self,
        command_buffer: &RafxCommandBuffer,
        imgui_draw_data: Option<&imgui::DrawData>,
    ) -> RafxResult<()> {
        let dpi_scaling = 1.0;
        let top = 0.0;
        let bottom = self.swapchain_helper.swapchain_def().height as f32 / dpi_scaling;
        let view_proj = glam::Mat4::orthographic_rh(
            0.0,
            self.swapchain_helper.swapchain_def().width as f32 / dpi_scaling,
            bottom,
            top,
            -100.0,
            100.0,
        );

        let device_context = self.resource_manager.device_context();
        let dyn_resource_allocator = self.resource_manager.create_dyn_resource_allocator_set();
        let mut vertex_buffers = Vec::default();
        let mut index_buffers = Vec::default();
        if let Some(draw_data) = imgui_draw_data {
            for draw_list in draw_data.draw_lists() {
                let vertex_buffer_size = draw_list.vtx_buffer().len() as u64
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
                    .copy_to_host_visible_buffer(draw_list.vtx_buffer())
                    .unwrap();
                let vertex_buffer = dyn_resource_allocator.insert_buffer(vertex_buffer);

                let index_buffer_size = draw_list.idx_buffer().len() as u64
                    * std::mem::size_of::<imgui::DrawIdx>() as u64;

                let index_buffer = device_context
                    .create_buffer(&RafxBufferDef {
                        size: index_buffer_size,
                        memory_usage: RafxMemoryUsage::CpuToGpu,
                        resource_type: RafxResourceType::INDEX_BUFFER,
                        ..Default::default()
                    })
                    .unwrap();

                index_buffer
                    .copy_to_host_visible_buffer(draw_list.idx_buffer())
                    .unwrap();
                let index_buffer = dyn_resource_allocator.insert_buffer(index_buffer);

                vertex_buffers.push(vertex_buffer);
                index_buffers.push(index_buffer);
            }

            let imgui_pipeline = self
                .resource_manager
                .graphics_pipeline_cache()
                .get_or_create_graphics_pipeline(
                    OpaqueRenderPhase::render_phase_index(),
                    &self.imgui_pass.material_pass_resource,
                    &GraphicsPipelineRenderTargetMeta::new(
                        vec![self.swapchain_helper.format()],
                        None,
                        RafxSampleCount::SampleCount1,
                    ),
                    &IMGUI_VERTEX_LAYOUT,
                )?;
            command_buffer.cmd_bind_pipeline(&*imgui_pipeline.get_raw().pipeline)?;

            let mut descriptor_set_allocator =
                self.resource_manager.create_descriptor_set_allocator();
            let mut descriptor_set = descriptor_set_allocator
                .create_dyn_descriptor_set_uninitialized(
                    &self
                        .imgui_pass
                        .material_pass_resource
                        .get_raw()
                        .descriptor_set_layouts[0],
                )?;

            descriptor_set.set_image(1, &self.font_atlas_texture);
            descriptor_set.set_buffer_data(2, &view_proj);

            descriptor_set.flush(&mut descriptor_set_allocator)?;
            descriptor_set_allocator.flush_changes()?;

            descriptor_set.bind(command_buffer)?;

            for (draw_list_index, draw_list) in draw_data.draw_lists().enumerate() {
                command_buffer.cmd_bind_vertex_buffers(
                    0,
                    &[RafxVertexBufferBinding {
                        buffer: &vertex_buffers[draw_list_index].get_raw().buffer,
                        offset: 0,
                    }],
                )?;

                command_buffer.cmd_bind_index_buffer(&RafxIndexBufferBinding {
                    buffer: &index_buffers[draw_list_index].get_raw().buffer,
                    offset: 0,
                    index_type: RafxIndexType::Uint16,
                })?;

                let mut element_begin_index: u32 = 0;
                for cmd in draw_list.commands() {
                    match cmd {
                        imgui::DrawCmd::Elements {
                            count,
                            cmd_params:
                                imgui::DrawCmdParams {
                                    clip_rect,
                                    //texture_id,
                                    ..
                                },
                        } => {
                            let element_end_index = element_begin_index + count as u32;

                            let scissor_x = ((clip_rect[0] - draw_data.display_pos[0])
                                * draw_data.framebuffer_scale[0])
                                as u32;

                            let scissor_y = ((clip_rect[1] - draw_data.display_pos[1])
                                * draw_data.framebuffer_scale[1])
                                as u32;

                            let scissor_width =
                                ((clip_rect[2] - clip_rect[0] - draw_data.display_pos[0])
                                    * draw_data.framebuffer_scale[0])
                                    as u32;

                            let scissor_height =
                                ((clip_rect[3] - clip_rect[1] - draw_data.display_pos[1])
                                    * draw_data.framebuffer_scale[1])
                                    as u32;

                            command_buffer.cmd_set_scissor(
                                scissor_x,
                                scissor_y,
                                scissor_width,
                                scissor_height,
                            )?;

                            command_buffer.cmd_draw_indexed(
                                element_end_index - element_begin_index,
                                element_begin_index,
                                0,
                            )?;

                            element_begin_index = element_end_index;
                        }
                        _ => panic!("unexpected draw command"),
                    }
                }
            }
        }

        Ok(())
    }

    fn load_material_pass(
        resource_context: &ResourceContext,
        cooked_vertex_shader_bytes: &[u8],
        cooked_fragment_shader_bytes: &[u8],
        fixed_function_state: FixedFunctionState,
    ) -> RafxResult<MaterialPass> {
        let cooked_vertex_shader_stage =
            bincode::deserialize::<CookedShaderPackage>(cooked_vertex_shader_bytes)
                .map_err(|x| format!("Failed to deserialize cooked shader: {:?}", x))?;

        let vertex_shader_module = resource_context
            .resources()
            .get_or_create_shader_module_from_cooked_package(&cooked_vertex_shader_stage)?;
        let vertex_entry_point = cooked_vertex_shader_stage
            .find_entry_point("main")
            .unwrap()
            .clone();

        // Create the fragment shader module and find the entry point
        let cooked_fragment_shader_stage =
            bincode::deserialize::<CookedShaderPackage>(cooked_fragment_shader_bytes)
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
        self.graphics_queue.wait_for_queue_idle().unwrap();
        log::debug!("destroyed Renderer");
    }
}

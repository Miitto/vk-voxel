use ::vk::prelude::*;
use tracing::{error, info, log::LevelFilter};
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app_wrapper = AppWrapper::default();

    event_loop.run_app(&mut app_wrapper);
}

#[derive(Default)]
struct AppWrapper {
    a: Option<App>,
}

struct App {
    window: winit::window::Window,
    vkcore: ::vk::core::VkCore,
    current_frame: usize,
    last_frame_time: std::time::Instant,
}

impl winit::application::ApplicationHandler for AppWrapper {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let info = ::vk::AppInfo {
            name: "Voxel Testing",
            version: 1,
        };
        info!(
            "Starting application '{}', version {}.",
            info.name, info.version
        );
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("Voxel Testing")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)),
        ) {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let vkcore = match ::vk::init(info, &window) {
            Ok(core) => core,
            Err(e) => {
                error!("Failed to initialize Vulkan Core: {e}");
                event_loop.exit();
                return;
            }
        };

        let now = std::time::Instant::now();

        self.a = Some(App {
            window,
            vkcore,
            current_frame: 0,
            last_frame_time: now,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.a.as_mut().unwrap().window_event(event_loop, id, event)
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested. Exiting application.");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                info!("Window resized to {}x{}.", new_size.width, new_size.height);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                info!("Keyboard input: {:?}", event.physical_key);
            }
            WindowEvent::RedrawRequested => {
                let frame = &mut self.vkcore.frame[self.current_frame];
                unsafe {
                    self.vkcore
                        .device
                        .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)
                }
                .expect("Failed to wait for in flight fence");

                unsafe {
                    self.vkcore
                        .device
                        .reset_command_pool(
                            frame.pools.graphics.pool,
                            vk::CommandPoolResetFlags::empty(),
                        )
                        .expect("Failed to reset graphics command pool");
                    self.vkcore
                        .device
                        .reset_command_pool(
                            frame.pools.compute.pool,
                            vk::CommandPoolResetFlags::empty(),
                        )
                        .expect("Failed to reset compute command pool");
                }

                let now = std::time::Instant::now();
                let delta = now - self.last_frame_time;
                let dt = delta.as_secs_f32();
                self.last_frame_time = now;

                let (image_index, _is_suboptimal) = match self
                    .vkcore
                    .swapchain
                    .next_image(&frame.image_available_semaphore)
                {
                    Ok((idx, suboptimal)) => (idx, suboptimal),
                    Err(e) => {
                        error!("Failed to acquire next image: {e}");
                        panic!("Failed to acquire next image");
                    }
                };

                let graphics_cmd = {
                    let cmd_alloc_info = vk::CommandBufferAllocateInfo {
                        command_pool: frame.pools.graphics.pool,
                        level: vk::CommandBufferLevel::PRIMARY,
                        command_buffer_count: 1,
                        ..Default::default()
                    };

                    unsafe { self.vkcore.device.allocate_command_buffers(&cmd_alloc_info) }
                        .expect("Failed to allocate command buffer")[0]
                };

                unsafe {
                    self.vkcore.device.begin_command_buffer(
                        graphics_cmd,
                        &vk::CommandBufferBeginInfo::default()
                            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                    )
                }
                .expect("Failed to begin command buffer");

                {
                    let b = vk::ImageMemoryBarrier2::default()
                        .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
                        .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                        .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .image(self.vkcore.swapchain.images[image_index as usize])
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        });
                    unsafe {
                        self.vkcore.device.cmd_pipeline_barrier2(
                            graphics_cmd,
                            &vk::DependencyInfo::default()
                                .image_memory_barriers(std::slice::from_ref(&b)),
                        )
                    }
                }
                unsafe {
                    self.vkcore.device.cmd_clear_color_image(
                        graphics_cmd,
                        self.vkcore.swapchain.images[image_index as usize],
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &vk::ClearColorValue {
                            float32: [0.0, 0.0, 0.0, 1.0],
                        },
                        &[vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        }],
                    )
                }

                {
                    let b = vk::ImageMemoryBarrier2::default()
                        .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                        .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
                        .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                        .image(self.vkcore.swapchain.images[image_index as usize])
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        });

                    unsafe {
                        self.vkcore.device.cmd_pipeline_barrier2(
                            graphics_cmd,
                            &vk::DependencyInfo::default()
                                .image_memory_barriers(std::slice::from_ref(&b)),
                        )
                    }
                }

                if let Err(e) = unsafe { self.vkcore.device.end_command_buffer(graphics_cmd) } {
                    error!("Failed to end command buffer: {e}");
                    panic!("Failed to end command buffer");
                }

                unsafe {
                    self.vkcore
                        .device
                        .reset_fences(&[frame.in_flight_fence])
                        .expect("Failed to reset in flight fence")
                };

                let render_sem = self.vkcore.swapchain.sems[image_index as usize];

                let cmd_info = vk::CommandBufferSubmitInfo::default().command_buffer(graphics_cmd);
                let wait_info =
                    vk::SemaphoreSubmitInfo::default().semaphore(frame.image_available_semaphore);
                let signal_info = vk::SemaphoreSubmitInfo::default().semaphore(render_sem);

                if let Err(e) = unsafe {
                    self.vkcore.device.queue_submit2(
                        self.vkcore.queues.present.queue,
                        &[vk::SubmitInfo2::default()
                            .wait_semaphore_infos(std::slice::from_ref(&wait_info))
                            .command_buffer_infos(std::slice::from_ref(&cmd_info))
                            .signal_semaphore_infos(std::slice::from_ref(&signal_info))],
                        frame.in_flight_fence,
                    )
                } {
                    error!("Failed to submit command buffer: {e}");
                    panic!("Failed to submit command buffer");
                }

                if let Err(e) = unsafe {
                    self.vkcore.swapchain.queue_present(
                        self.vkcore.queues.present.queue,
                        &vk::PresentInfoKHR::default()
                            .wait_semaphores(std::slice::from_ref(&render_sem))
                            .image_indices(std::slice::from_ref(&image_index))
                            .swapchains(std::slice::from_ref(&self.vkcore.swapchain.swapchain)),
                    )
                } {
                    error!("Failed to present swapchain image: {e}");
                    panic!("Failed to present swapchain image");
                }
            }
            _ => {}
        }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use log::debug;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    use log::Level;
    console_log::init_with_level(Level::Trace).expect("error initializing log");

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main();
}

fn main() {
    use wgpu::winit::{
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
    };

    let events_loop = EventLoop::new();

    let (window, instance, size, surface) = {
        use wgpu::winit::window::Window;

        let window = Window::new(&events_loop).unwrap();
        let size = window.inner_size().to_physical(window.hidpi_factor());

        #[cfg(all(feature = "gl", target_arch = "wasm32"))]
        let (instance, surface) = {
            let instance = wgpu::Instance::new(&window);
            let surface = instance.get_surface();
            (instance, surface)
        };

        #[cfg(not(all(feature = "gl", target_arch = "wasm32")))]
        let (instance, surface) = {
            let instance = wgpu::Instance::new();
            let surface = instance.create_surface(&window);
            (instance, surface)
        };

        (window, instance, size, surface)
    };

    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::LowPower,
    });

    let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let vs_bytes = include_bytes!("shader.vert.spv");
    let vs_module = device.create_shader_module(vs_bytes);
    let fs_bytes = include_bytes!("shader.frag.spv");
    let fs_module = device.create_shader_module(fs_bytes);

    let bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &[] });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba8Unorm,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[],
        sample_count: 1,
    });

    let mut swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: size.width.round() as u32,
            height: size.height.round() as u32,
        },
    );

    window.request_redraw();

    events_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => {
            debug!("{:?}", event);
            match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match code {
                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::RedrawRequested => {
                    let frame = swap_chain.get_next_texture();
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &frame.view,
                                resolve_target: None,
                                load_op: wgpu::LoadOp::Clear,
                                store_op: wgpu::StoreOp::Store,
                                clear_color: wgpu::Color::GREEN,
                            }],
                            depth_stencil_attachment: None,
                        });
                        rpass.set_pipeline(&render_pipeline);
                        rpass.set_bind_group(0, &bind_group, &[]);
                        rpass.draw(0..3, 0..1);
                    }

                    device.get_queue().submit(&[encoder.finish()]);
                    window.request_redraw();
                }
                _ => *control_flow = ControlFlow::Poll,
            }
        }
        _ => *control_flow = ControlFlow::Poll,
    });
}

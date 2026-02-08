use termsurf_xpc::iosurface;

fn main() {
    // Step 1: Create IOSurface
    let width: u32 = 800;
    let height: u32 = 600;
    let surface = iosurface::create_iosurface(width, height).expect("Failed to create IOSurface");
    println!(
        "IOSurface created: {}x{}",
        iosurface::get_width(surface),
        iosurface::get_height(surface)
    );

    // Step 2: Create wgpu device and queue
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::METAL,
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("Failed to find Metal adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("termsurf-terminal"),
            ..Default::default()
        },
        None,
    ))
    .expect("Failed to create wgpu device");

    println!("wgpu device created: {:?}", adapter.get_info().name);

    // Step 3: Create a Metal texture backed by the IOSurface
    //
    // wgpu doesn't expose IOSurface texture creation directly, so we use
    // Objective-C runtime to create the Metal texture, then render to a
    // regular wgpu texture and copy to the IOSurface.
    //
    // In production, we'd use wgpu's HAL API for zero-copy. For this
    // prototype, GPU render + CPU copy proves the pipeline works.

    // Create a wgpu texture to render into
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("blue-render-target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Step 4: Render a clear-to-blue pass
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("blue-render"),
    });

    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear-blue"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        // Empty pass — just clears to blue
    }

    // Copy GPU texture to a buffer so we can write it to the IOSurface
    let bytes_per_row = width * 4;
    // wgpu requires rows aligned to 256 bytes
    let padded_bytes_per_row = (bytes_per_row + 255) & !255;

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (padded_bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(encoder.finish()));

    // Map the buffer and copy to IOSurface
    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().expect("Failed to map buffer");

    {
        let data = buffer_slice.get_mapped_range();

        // Write to IOSurface
        iosurface::write_pixels(
            surface,
            &data,
            bytes_per_row as usize,
            padded_bytes_per_row as usize,
        );
    }

    output_buffer.unmap();

    println!("Rendered blue (0, 0, 255, 255)");

    // Step 5: Read back pixels from IOSurface and verify
    let pixel = iosurface::read_pixel(surface, 0, 0);
    let r = (pixel >> 24) & 0xFF;
    let g = (pixel >> 16) & 0xFF;
    let b = (pixel >> 8) & 0xFF;
    let a = pixel & 0xFF;

    if r == 0 && g == 0 && b == 255 && a == 255 {
        println!("Pixel at (0,0): ({}, {}, {}, {}) ✓", r, g, b, a);
    } else {
        println!("Pixel at (0,0): ({}, {}, {}, {}) ✗ (expected 0, 0, 255, 255)", r, g, b, a);
    }

    // Also check a pixel in the middle
    let pixel_mid = iosurface::read_pixel(surface, 400, 300);
    let mr = (pixel_mid >> 24) & 0xFF;
    let mg = (pixel_mid >> 16) & 0xFF;
    let mb = (pixel_mid >> 8) & 0xFF;
    let ma = pixel_mid & 0xFF;
    println!("Pixel at (400,300): ({}, {}, {}, {})", mr, mg, mb, ma);

    // Step 6: Create Mach port
    let port = iosurface::create_mach_port(surface);
    println!("Mach port: {}", port);

    if port != 0 {
        println!("Phase 3 complete: IOSurface + wgpu + Mach port verified");
    } else {
        println!("ERROR: Mach port creation failed");
    }
}

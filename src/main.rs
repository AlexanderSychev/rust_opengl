use glow::*;
use glutin;
use std::sync::Arc;

mod logging;
mod metadata;
mod shader;

use nalgebra_glm::Vec4;

/// Блок uniform-переменных в виде структуры
#[derive(Debug, Clone)]
struct BlobSettings {
    outer_color: Vec4,
    inner_color: Vec4,
    radius_inner: f32,
    radius_outer: f32,
}

fn init_log() {
    use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}

fn main() {
    init_log();

    use bytemuck::cast_slice;

    // Координаты вершин квадрата (в нормализованной форме)
    #[rustfmt::skip]
    let vertex_position: Vec<f32> = {
        vec![
            -0.8, -0.8, 0.0,
            0.8, -0.8, 0.0,
            0.8,  0.8, 0.0,
            -0.8, -0.8, 0.0,
            0.8,  0.8, 0.0,
            -0.8,  0.8, 0.0,
        ]
    };

    // Текстурные координаты вершин квадрата (в нормализованной форме)
    #[rustfmt::skip]
    let vertex_tex_coord: Vec<f32> = {
        vec![
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 0.0,
            1.0, 1.0,
            0.0, 1.0
        ]
    };

    use nalgebra_glm::vec4;
    let blob_settings = BlobSettings {
        outer_color: vec4(0.0, 0.0, 0.0, 0.0),
        inner_color: vec4(1.0f32, 1.0f32, 0.75f32, 1.0f32),
        radius_inner: 0.25,
        radius_outer: 0.45,
    };

    unsafe {
        let (gl, window, event_loop) = {
            use logging::gl_log_callback;
            let event_loop = glutin::event_loop::EventLoop::new();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("Rust OpenGL Learning Sandbox")
                .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();
            let mut gl =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            gl.enable(glow::DEBUG_OUTPUT);
            gl.debug_message_callback(gl_log_callback);
            gl.debug_message_control(
                glow::DONT_CARE,
                glow::DONT_CARE,
                glow::DONT_CARE,
                &vec![],
                true,
            );
            (Arc::new(gl), window, event_loop)
        };

        // Проверяем, что видеокарта поддерживает OpenGL 4.3+
        let gl_metadata = metadata::OpenGlMetadata::from(gl.clone());
        gl_metadata.assert_version();
        println!("{:?}", gl_metadata);

        // Создать и заполнить буффер координат
        let position_buffer = gl.create_buffer().ok();
        gl.bind_buffer(glow::ARRAY_BUFFER, position_buffer);
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            cast_slice(&vertex_position),
            glow::STATIC_DRAW,
        );

        // Создать и заполнить буффер цветов
        let tex_coord_buffer = gl.create_buffer().ok();
        gl.bind_buffer(glow::ARRAY_BUFFER, tex_coord_buffer);
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            cast_slice(&vertex_tex_coord),
            glow::STATIC_DRAW,
        );

        // Создать объект массива вершин
        let vertex_array = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        // Активировать массивы вершинных атрибутов
        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);

        // Закрепить индекс 0 за буфером с координатами
        gl.bind_buffer(glow::ARRAY_BUFFER, position_buffer);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

        // Закрепить индекс 1 за буфером с цветами
        gl.bind_buffer(glow::ARRAY_BUFFER, tex_coord_buffer);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);

        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        let mut program = shader::ShaderProgram::new(gl.clone()).unwrap();

        program
            .compile_shader("shaders/vertex.glsl", shader::ShaderType::Vertex)
            .unwrap();
        program
            .compile_shader("shaders/fragment.glsl", shader::ShaderType::Fragment)
            .unwrap();

        program.link().unwrap();

        program.print_active_attribs();
        program.print_active_uniforms();

        program.use_program().unwrap();

        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;

        event_loop.run(move |event, _, control_flow| match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                use shader::GlslValue;

                gl.clear(glow::COLOR_BUFFER_BIT);

                gl.bind_vertex_array(Some(vertex_array));
                gl.draw_arrays(glow::TRIANGLES, 0, 6);

                program.set_uniform_value(
                    "inner_color",
                    GlslValue::Float32Vec4(blob_settings.inner_color),
                );
                program.set_uniform_value(
                    "outer_color",
                    GlslValue::Float32Vec4(blob_settings.outer_color),
                );
                program.set_uniform_value(
                    "radius_inner",
                    GlslValue::Float32(blob_settings.radius_inner),
                );
                program.set_uniform_value(
                    "radius_outer",
                    GlslValue::Float32(blob_settings.radius_outer),
                );

                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                }
                WindowEvent::CloseRequested => {
                    // gl.delete_program(program.get_handle());
                    gl.delete_vertex_array(vertex_array);
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            _ => (),
        });
    }
}

use crate::geometry::Drawable;
use glow::*;
use glutin;
use std::sync::Arc;

mod geometry;
mod logging;
mod metadata;
mod shader;

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

use nalgebra_glm::Mat4;

fn init_model() -> (Mat4, Mat4, Mat4) {
    use nalgebra_glm::{look_at, mat4, rotate, vec3};

    #[rustfmt::skip]
    let mut model = {
        mat4::<f32>(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    };
    model = rotate(&model, -35.0, &vec3(1.0, 0.0, 0.0));
    model = rotate(&model, 35.0, &vec3(0.0, 1.0, 0.0));

    let view = look_at::<f32>(
        &vec3(0.0, 0.0, 2.0),
        &vec3(0.0, 0.0, 0.0),
        &vec3(0.0, 1.0, 0.0),
    );

    #[rustfmt::skip]
    let projection = {
        mat4::<f32>(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    };

    (model, view, projection)
}

fn main() {
    init_log();
    use geometry::TriangleMesh;

    let (model, view, projection) = init_model();

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

        let torus = TriangleMesh::new_torus(gl.clone(), 0.7, 0.3, 30, 30).unwrap();

        let shader_manager = {
            let mut sm = shader::ShaderManager::new(gl.clone());
            sm.load_shader(
                "vertex",
                "shaders/light/vertex.glsl",
                shader::ShaderType::Vertex,
            )
            .unwrap();
            sm.load_shader(
                "fragment",
                "shaders/light/fragment.glsl",
                shader::ShaderType::Fragment,
            )
            .unwrap();
            Arc::new(sm)
        };

        let mut program = shader::ShaderProgram::new(gl.clone(), shader_manager.clone()).unwrap();
        program.attach_shader("vertex");
        program.attach_shader("fragment");

        program.link().unwrap();

        program.print_active_attribs();
        program.print_active_uniforms();

        program.use_program().unwrap();

        program.set_uniform_value(
            "kd",
            shader::GlslValue::Float32Vec3(nalgebra_glm::vec3(0.9, 0.5, 0.3)),
        );
        program.set_uniform_value(
            "ld",
            shader::GlslValue::Float32Vec3(nalgebra_glm::vec3(1.0, 1.0, 1.0)),
        );
        program.set_uniform_value(
            "light_position",
            shader::GlslValue::Float32Vec4(nalgebra_glm::vec4(5.0, 5.0, 2.0, 1.0)),
        );

        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;

        event_loop.run(move |event, _, control_flow| match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                gl.clear(glow::COLOR_BUFFER_BIT);

                let model_view_matrix = view * model;
                #[rustfmt::skip]
                let normal_matrix = {
                    nalgebra_glm::mat3(
                        model_view_matrix.row(0)[0], model_view_matrix.row(0)[1], model_view_matrix.row(0)[2],
                        model_view_matrix.row(1)[0], model_view_matrix.row(1)[1], model_view_matrix.row(1)[2],
                        model_view_matrix.row(2)[0], model_view_matrix.row(2)[1], model_view_matrix.row(2)[2],
                    )
                };
                program.set_uniform_value(
                    "model_view_matrix",
                    shader::GlslValue::Float32Mat4(model_view_matrix),
                );
                program.set_uniform_value(
                    "normal_matrix",
                    shader::GlslValue::Float32Mat3(normal_matrix),
                );
                program.set_uniform_value(
                    "mvp",
                    shader::GlslValue::Float32Mat4(projection * model_view_matrix),
                );
                torus.render();

                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                }
                WindowEvent::CloseRequested => {
                    // gl.delete_program(program.get_handle());
                    // gl.delete_vertex_array(vertex_array);
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            _ => (),
        });
    }
}

use glow::*;
use glutin;
use std::sync::Arc;

mod metadata;

fn main() {
    use bytemuck::cast_slice;

    // Координаты вершин треугольника (в нормализованной форме)
    let position_data: Vec<f32> = vec![-0.8, -0.8, 0.0, 0.8, -0.8, 0.0, 0.0, 0.8, 0.0];

    // Цвета вершин треугольника (в нормализованной форме)
    let colors_data: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    let mut angle: f32 = 0.0;

    unsafe {
        let (gl, window, event_loop) = {
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
            let gl =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            (Arc::new(gl), window, event_loop)
        };

        // Проверяем, что видеокарта поддерживает OpenGL 4.3+
        let gl_metadata = metadata::OpenGlMetadata::from(gl.clone());
        gl_metadata.assert_version();
        println!("{:?}", gl_metadata);

        // Создать и заполнить буффер координат
        let position_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(position_buffer));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            cast_slice(&position_data),
            glow::STATIC_DRAW,
        );

        // Создать и заполнить буффер цветов
        let color_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(color_buffer));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            cast_slice(&colors_data),
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
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(position_buffer));
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

        // Закрепить индекс 1 за буфером с цветами
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(color_buffer));
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0);

        let program = gl.create_program().expect("Cannot create program");

        // Привязать индексы к входным переменным вершинного шейдера (вместо "layout (location = <index>)")
        gl.bind_attrib_location(program, 0, "vertex_position");
        gl.bind_attrib_location(program, 1, "vertex_color");

        // Привязать индекс в выходной переменной фрагментного шейдера (вместо "layout (location = <index>)")
        gl.bind_frag_data_location(program, 0, "frag_color");

        let (vertex_shader_source, fragment_shader_source) = {
            use std::fs::read_to_string;
            (
                read_to_string("shaders/vertex.glsl").unwrap(),
                read_to_string("shaders/fragment.glsl").unwrap(),
            )
        };

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, shader_source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        // Получить число входных атрибутов
        let num_attribs = gl.get_active_attributes(program);
        println!("Active attributes count: {}", num_attribs);

        // Вывести все найденные атрибуты
        if num_attribs > 0 {
            for i in 0..num_attribs {
                let maybe_attr = gl.get_active_attribute(program, i);
                if let Some(attr) = maybe_attr {
                    println!("#{} - \"{}\" {}", i, attr.name, attr.size);
                } else {
                    println!("#{} - None", i);
                }
            }
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));
        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;

        event_loop.run(move |event, _, control_flow| match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                angle += 0.0174533;
                gl.clear(glow::COLOR_BUFFER_BIT);

                if let Some(uniform) = gl.get_uniform_location(program, "rotation_matrix") {
                    use nalgebra_glm::{rotate, mat4, vec3};
                    let rotation_matrix = rotate(
                        &mat4::<f32>(
                            1.0, 0.0, 0.0, 0.0,
                            0.0, 1.0, 0.0, 0.0,
                            0.0, 0.0, 1.0, 0.0,
                            0.0, 0.0, 0.0, 1.0,
                        ),
                        angle,
                        &vec3::<f32>(0.0, 0.0, 1.0),
                    );

                    gl.uniform_matrix_4_f32_slice(Some(&uniform), false, rotation_matrix.as_slice());
                }

                gl.bind_vertex_array(Some(vertex_array));
                gl.draw_arrays(glow::TRIANGLES, 0, 3);

                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                }
                WindowEvent::CloseRequested => {
                    gl.delete_program(program);
                    gl.delete_vertex_array(vertex_array);
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            _ => (),
        });
    }
}

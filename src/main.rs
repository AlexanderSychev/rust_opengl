use glow::*;
use glutin;
use std::sync::Arc;

mod metadata;
mod log;

/// Блок uniform-переменных в виде структуры
#[derive(Debug, Clone)]
struct BlobSettings {
    outer_color: Vec<f32>,
    inner_color: Vec<f32>,
    inner_radius: f32,
    outer_radius: f32,
}

impl BlobSettings {
    /// Преобразовать структуру в срез байтов (для передачи в OpenGL)
    pub fn as_bytes(&self) -> Vec<u8> {
        use bytemuck::cast_slice;
        let mut outer_color = Vec::from(cast_slice::<f32, u8>(&self.outer_color));
        let mut inner_color = Vec::from(cast_slice::<f32, u8>(&self.inner_color));
        let mut inner_radius = Vec::from(self.inner_radius.to_be_bytes().as_slice());
        let mut outer_radius = Vec::from(self.outer_radius.to_be_bytes().as_slice());
        
        let mut result = Vec::with_capacity(
            outer_color.len() +
            inner_color.len() +
            inner_radius.len() +
            outer_radius.len()
        );
        result.append(&mut outer_color);
        result.append(&mut inner_color);
        result.append(&mut inner_radius);
        result.append(&mut outer_radius);

        result
    }
}

fn main() {
    use bytemuck::cast_slice;

    // Координаты вершин квадрата (в нормализованной форме)
    let position_data: Vec<f32> = vec![
        -0.5, -0.5, 0.0,
        -0.5, 0.5, 0.0,
        0.5, -0.5, 0.0,
        0.5, 0.5, 0.0,
    ];

    // Цвета вершин квадрата (в нормализованной форме)
    let colors_data: Vec<f32> = vec![
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
        0.5, 0.5, 0.5,
    ];

    let blob_settings = BlobSettings {
        outer_color: vec![0.0f32; 4],
        inner_color: vec![1.0f32, 1.0f32, 0.75f32, 1.0f32],
        inner_radius: 0.25,
        outer_radius: 0.45,
    };

    unsafe {
        let (gl, window, event_loop) = {
            use log::gl_log_callback;
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
            let mut gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
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
        for i in 0..num_attribs {
            let maybe_attr = gl.get_active_attribute(program, i);
            if let Some(attr) = maybe_attr {
                println!("#{} - \"{}\" {}", i, attr.name, attr.size);
            } else {
                println!("#{} - None", i);
            }
        }

        // Получить число всех uniform-переменных
        let num_uniforms = gl.get_active_uniforms(program);
        println!("Uniforms count: {}", num_uniforms);

        // Вывести все найденные uniform-переменные
        for i in 0..num_uniforms {
            let maybe_uniform = gl.get_active_uniform(program, i);
            if let Some(uniform) = maybe_uniform {
                println!("#{} - \"{}\" {}", i, uniform.name, uniform.size);
            } else {
                println!("#{} - None", i);
            }
        }

        // Получаем индекс uniform-блока и его размер
        let block_index = gl.get_uniform_block_index(program, "blob_settings").unwrap();
        let block_size = gl.get_active_uniform_block_parameter_i32(program, block_index, glow::UNIFORM_BLOCK_DATA_SIZE) as usize;
        println!("block_size = {}", block_size);

        // Создаем буфферный объект и копируем в него данные
        let uniform_buffer = gl.create_buffer().ok();
        gl.bind_buffer(glow::UNIFORM_BUFFER, uniform_buffer);
        gl.buffer_data_u8_slice(glow::UNIFORM_BUFFER, &blob_settings.as_bytes(), glow::DYNAMIC_DRAW);

        gl.bind_buffer_base(glow::UNIFORM_BUFFER, 0, uniform_buffer);

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
                gl.clear(glow::COLOR_BUFFER_BIT);

                gl.bind_vertex_array(Some(vertex_array));
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
                gl.draw_arrays(glow::TRIANGLES, 1, 3);

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

use core::f32;
use glow::{Buffer, Context, HasContext, VertexArray};
use simple_error::{SimpleError, SimpleResult};
use std::sync::Arc;

pub trait Drawable {
    fn render(&self);
}

#[derive(Debug)]
pub struct TriangleMesh {
    context: Arc<Context>,
    vertex_count: i32,
    vertex_array: VertexArray,
    buffers: Vec<Buffer>,
}

impl TriangleMesh {
    pub fn new_torus(
        context: Arc<Context>,
        outer_radius: f32,
        inner_raduis: f32,
        num_sides: usize,
        num_rings: usize,
    ) -> SimpleResult<TriangleMesh> {
        use nalgebra_glm::two_pi;

        let faces = num_sides * num_rings;
        let num_verts = num_sides * (num_rings + 1); // One extra ring to duplicate first ring

        let mut points: Vec<f32> = vec![0.0; 3 * num_verts];
        let mut normals: Vec<f32> = vec![0.0; 3 * num_verts];
        let mut tex_coords: Vec<f32> = vec![0.0; 2 * num_verts];
        let mut indicies: Vec<u32> = vec![0; 6 * faces];

        let ring_factor: f32 = two_pi::<f32>() / (num_rings as f32);
        let side_factor: f32 = two_pi::<f32>() / (num_sides as f32);

        let mut idx: usize = 0;
        let mut tidx: usize = 0;
        for ring in 0..(num_rings + 1) {
            let u = ring_factor * (ring as f32);
            let cu = u.cos();
            let su = u.sin();
            for side in 0..num_sides {
                let v = side_factor * (side as f32);
                let cv = v.cos();
                let sv = v.sin();
                let r = outer_radius + inner_raduis * cv;

                points[idx] = r * cu;
                points[idx + 1] = r * su;
                points[idx + 2] = inner_raduis * sv;

                normals[idx] = cv * cu * r;
                normals[idx + 1] = cv * su * r;
                normals[idx + 2] = sv * r;

                tex_coords[tidx] = u / two_pi::<f32>();
                tex_coords[tidx + 1] = v / two_pi::<f32>();
                tidx += 2;

                // Normalize
                let len = (normals[idx].powf(2.0)
                    + normals[idx + 1].powf(2.0)
                    + normals[idx + 2].powf(2.0))
                .sqrt();
                normals[idx] /= len;
                normals[idx + 1] /= len;
                normals[idx + 2] /= len;
                idx += 3;
            }
        }

        idx = 0;
        for ring in 0..num_rings {
            let ring_start = ring * num_sides;
            let next_ring_start = (ring + 1) * num_sides;
            for side in 0..num_sides {
                let next_side = (side + 1) % num_sides;
                indicies[idx] = (ring_start + side) as u32;
                indicies[idx + 1] = (next_ring_start + side) as u32;
                indicies[idx + 2] = (next_ring_start + next_side) as u32;
                indicies[idx + 3] = (ring_start + side) as u32;
                indicies[idx + 4] = (next_ring_start + next_side) as u32;
                indicies[idx + 5] = (ring_start + next_side) as u32;
                idx += 6;
            }
        }

        return TriangleMesh::new(context, indicies, points, normals, Some(tex_coords), None);
    }

    pub fn new(
        context: Arc<Context>,
        indices: Vec<u32>,                  // Индексы
        points: Vec<f32>,                   // Точки
        normals: Vec<f32>,                  // Нормали
        maybe_tex_coords: Option<Vec<f32>>, // Текстурные координаты (необязательно)
        maybe_tangents: Option<Vec<f32>>,   // Касательные (необязательно)
    ) -> SimpleResult<TriangleMesh> {
        let vertex_count = indices.len() as i32;
        let mut buffers: Vec<Buffer> = vec![];

        use bytemuck::cast_slice;
        use glow::{ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FLOAT, STATIC_DRAW};

        let index_buffer = unsafe {
            match context.create_buffer() {
                Ok(buffer) => Ok(buffer),
                Err(err) => Err(SimpleError::new(err)),
            }
        }?;
        buffers.push(index_buffer);
        unsafe {
            context.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            context.buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, cast_slice(&indices), STATIC_DRAW);
        }

        let position_buffer = unsafe {
            match context.create_buffer() {
                Ok(buffer) => Ok(buffer),
                Err(err) => Err(SimpleError::new(err)),
            }
        }?;
        buffers.push(position_buffer);
        unsafe {
            context.bind_buffer(ARRAY_BUFFER, Some(position_buffer));
            context.buffer_data_u8_slice(ARRAY_BUFFER, cast_slice(&points), STATIC_DRAW);
        }

        let normal_buffer = unsafe {
            match context.create_buffer() {
                Ok(buffer) => Ok(buffer),
                Err(err) => Err(SimpleError::new(err)),
            }
        }?;
        buffers.push(normal_buffer);
        unsafe {
            context.bind_buffer(ARRAY_BUFFER, Some(normal_buffer));
            context.buffer_data_u8_slice(ARRAY_BUFFER, cast_slice(&normals), STATIC_DRAW);
        }

        let maybe_text_coords_buffer: Option<Buffer> = if let Some(tex_coords) = maybe_tex_coords {
            let text_coords_buffer = unsafe {
                match context.create_buffer() {
                    Ok(buffer) => Ok(buffer),
                    Err(err) => Err(SimpleError::new(err)),
                }
            }?;
            buffers.push(text_coords_buffer);
            unsafe {
                context.bind_buffer(ARRAY_BUFFER, Some(text_coords_buffer));
                context.buffer_data_u8_slice(ARRAY_BUFFER, cast_slice(&tex_coords), STATIC_DRAW);
            }
            Some(text_coords_buffer)
        } else {
            None
        };

        let maybe_tangents_buffer = if let Some(tangents) = maybe_tangents {
            let tangents_buffer = unsafe {
                match context.create_buffer() {
                    Ok(buffer) => Ok(buffer),
                    Err(err) => Err(SimpleError::new(err)),
                }
            }?;
            buffers.push(tangents_buffer);
            unsafe {
                context.bind_buffer(ARRAY_BUFFER, Some(tangents_buffer));
                context.buffer_data_u8_slice(ARRAY_BUFFER, cast_slice(&tangents), STATIC_DRAW);
            }
            Some(tangents_buffer)
        } else {
            None
        };

        let vertex_array = unsafe {
            match context.create_vertex_array() {
                Ok(vertex_array) => Ok(vertex_array),
                Err(err) => Err(SimpleError::new(err)),
            }
        }?;
        unsafe {
            context.bind_vertex_array(Some(vertex_array));
            context.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(index_buffer));
        };

        // Vertex position
        unsafe {
            context.bind_buffer(ARRAY_BUFFER, Some(position_buffer));
            context.vertex_attrib_pointer_f32(0, 3, FLOAT, false, 0, 0);
            context.enable_vertex_attrib_array(0);
        }

        // Normal
        unsafe {
            context.bind_buffer(ARRAY_BUFFER, Some(normal_buffer));
            context.vertex_attrib_pointer_f32(1, 3, FLOAT, false, 0, 0);
            context.enable_vertex_attrib_array(1);
        }

        // Tex coords
        if let Some(text_coords_buffer) = maybe_text_coords_buffer {
            unsafe {
                context.bind_buffer(ARRAY_BUFFER, Some(text_coords_buffer));
                context.vertex_attrib_pointer_f32(2, 2, FLOAT, false, 0, 0);
                context.enable_vertex_attrib_array(2);
            }
        }

        // Tangents
        if let Some(tangents_buffer) = maybe_tangents_buffer {
            unsafe {
                context.bind_buffer(ARRAY_BUFFER, Some(tangents_buffer));
                context.vertex_attrib_pointer_f32(3, 4, FLOAT, false, 0, 0);
                context.enable_vertex_attrib_array(3);
            }
        }

        Ok(TriangleMesh {
            context,
            vertex_array,
            vertex_count,
            buffers,
        })
    }

    pub fn get_vertex_array(&self) -> VertexArray {
        self.vertex_array
    }

    fn delete_buffers(&mut self) {
        for buffer in &self.buffers {
            unsafe { self.context.delete_buffer(*buffer) };
        }
        self.buffers.clear();
    }
}

impl Drawable for TriangleMesh {
    fn render(&self) {
        use glow::{TRIANGLES, UNSIGNED_INT};

        unsafe {
            self.context.bind_vertex_array(Some(self.vertex_array));
            self.context
                .draw_elements(TRIANGLES, self.vertex_count, UNSIGNED_INT, 0);
        };
    }
}

impl Drop for TriangleMesh {
    fn drop(&mut self) {
        self.delete_buffers();
    }
}

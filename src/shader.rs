use glow::{
    Context, HasContext, Program, UniformLocation, Shader, COMPUTE_SHADER, FRAGMENT_SHADER,
    GEOMETRY_SHADER, TESS_CONTROL_SHADER, TESS_EVALUATION_SHADER, VERTEX_SHADER,
};
use nalgebra_glm::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use simple_error::SimpleError;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::borrow::Borrow;

// -----------------------------------------------------------------------------
// Shader type enumeration
// -----------------------------------------------------------------------------

/// Available GLSL shaders types
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    TessControl,
    TessEvaluation,
    Compute,
}

// Shader type enumeration value can be converted to
// native OpenGL shader type constant by standard `.into()` method
impl Into<u32> for ShaderType {
    fn into(self) -> u32 {
        match self {
            ShaderType::Vertex => VERTEX_SHADER,
            ShaderType::Fragment => FRAGMENT_SHADER,
            ShaderType::Geometry => GEOMETRY_SHADER,
            ShaderType::TessControl => TESS_CONTROL_SHADER,
            ShaderType::TessEvaluation => TESS_EVALUATION_SHADER,
            ShaderType::Compute => COMPUTE_SHADER,
        }
    }
}

// Shader type enumeration value can be created from
// native OpenGL shader type constant by standard `.try_from(value)` and
// `.from(value)` methods
impl TryFrom<u32> for ShaderType {
    type Error = SimpleError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            VERTEX_SHADER => Ok(ShaderType::Vertex),
            FRAGMENT_SHADER => Ok(ShaderType::Fragment),
            GEOMETRY_SHADER => Ok(ShaderType::Geometry),
            TESS_CONTROL_SHADER => Ok(ShaderType::TessControl),
            TESS_EVALUATION_SHADER => Ok(ShaderType::TessEvaluation),
            COMPUTE_SHADER => Ok(ShaderType::Compute),
            _ => Err(SimpleError::new(format!("Unknown shader type: {}", value))),
        }
    }
}

// -----------------------------------------------------------------------------
// GLSL values utils
// -----------------------------------------------------------------------------

/// Converts a native OpenGl constant describing a GLSL data type to
/// a string representing the corresponding keyword of that type.
/// Can be used for logging or code generation.
pub fn native_gl_value_type_to_keyword(native: u32) -> &'static str {
    use glow::{
        BOOL, DOUBLE, FLOAT, FLOAT_MAT2, FLOAT_MAT3, FLOAT_MAT4, FLOAT_VEC2, FLOAT_VEC3,
        FLOAT_VEC4, INT, UNSIGNED_INT,
    };
    match native {
        FLOAT => "float",
        FLOAT_VEC2 => "vec2",
        FLOAT_VEC3 => "vec3",
        FLOAT_VEC4 => "vec4",
        DOUBLE => "double",
        INT => "int",
        UNSIGNED_INT => "unsigned int",
        BOOL => "bool",
        FLOAT_MAT2 => "mat2",
        FLOAT_MAT3 => "mat3",
        FLOAT_MAT4 => "mat4",
        _ => "?",
    }
}

/// An algebraic data type that defines a value for GLSL.
/// Note that names closer to `Rust` than to `C` are used.
#[derive(Clone, Copy, Debug)]
pub enum GlslValue {
    /// 32-bit float value - `float` in GLSL , described by `GL_FLOAT` OpenGL constant
    Float32(f32),
    /// Two-dimensional vector of 32-bit float values - `vec2` in GLSL , described by `GL_FLOAT_VEC2` OpenGL constant.
    Float32Vec2(Vec2),
    /// Three-dimensional vector of 32-bit float values - `vec3` in GLSL , described by `GL_FLOAT_VEC3` OpenGL constant.
    Float32Vec3(Vec3),
    /// Four-dimensional vector of 32-bit float values - `vec4` in GLSL , described by `GL_FLOAT_VEC4` OpenGL constant.
    Float32Vec4(Vec4),
    /// 64-bit float value - `double` in GLSL , described by `GL_DOUBLE` OpenGL constant
    Float64(f64),
    // 32-bit integer value - `int` in GLSL, described by `GL_INT` OpenGL constant
    Int32(i32),
    // 32-bit unsigned integer value - `unsigned int` in GLSL, described by `GL_UNSIGNED_INT` OpenGL constant
    UnsignedInt32(u32),
    // Boolean value - `bool` in GLSL, described by `GL_BOOL` OpenGL constant
    Bool(bool),
    // 2x2 matrix of 32-bit float values - `mat2` in GLSL, described by `GL_FLOAT_MAT2` OpenGL constant
    Float32Mat2(Mat2),
    // 3x3 matrix of 32-bit float values - `mat3` in GLSL, described by `GL_FLOAT_MAT3` OpenGL constant
    Float32Mat3(Mat3),
    // 4x4 matrix of 32-bit float values - `mat4` in GLSL, described by `GL_FLOAT_MAT4` OpenGL constant
    Float32Mat4(Mat4),
}

// -----------------------------------------------------------------------------
// Shader manager
// -----------------------------------------------------------------------------


pub struct ShaderManager {
    /// OpenGL global context reference
    context: Arc<Context>,
    // Loaded and compiled shaders
    shaders: BTreeMap<String, Shader>,
}

impl ShaderManager {
    pub fn new(context: Arc<Context>) -> ShaderManager {
        ShaderManager {
            context,
            shaders: BTreeMap::new(),
        }
    }

    pub fn load_shader<P, Q>(
        &mut self,
        key: Q,
        filename: P,
        shader_type: ShaderType,
    ) -> Result<(), SimpleError>
    where
        P: AsRef<std::path::Path>,
        String: From<Q>,
    {
        // Read shader file
        use std::fs::read_to_string;
        let maybe_source = read_to_string(filename);
        if let Err(err) = maybe_source {
            return Err(SimpleError::from(err));
        }
        let source = maybe_source.unwrap();

        // Create shader with received type
        let maybe_shader = unsafe { self.context.create_shader(shader_type.into()) };
        if let Err(err) = maybe_shader {
            return Err(SimpleError::new(err));
        }
        let shader = maybe_shader.unwrap();

        // Compile shader
        let compile_succeed = unsafe {
            self.context.shader_source(shader, &source);
            self.context.compile_shader(shader);
            self.context.get_shader_compile_status(shader)
        };
        if !compile_succeed {
            return Err(SimpleError::new(unsafe {
                self.context.get_shader_info_log(shader)
            }));
        }

        self.shaders.insert(String::from(key), shader);

        Ok(())
    }

    pub fn has_shader<Q: ?Sized>(&self, key: &Q) -> bool
    where
        String : Borrow<Q> + Ord,
        Q: Ord,
    {
        self.shaders.contains_key(key)
    }

    pub fn get_shader<Q: ?Sized>(&self, key: &Q) -> Option<&Shader>
    where
        String : Borrow<Q> + Ord,
        Q: Ord,
    {
        self.shaders.get(key)
    }

    pub fn unload_shader<Q: ?Sized>(&mut self, key: &Q)
    where
        String : Borrow<Q> + Ord,
        Q: Ord,
    {
        let maybe_shader = self.shaders.get(key);
        if let Some(shader) = maybe_shader {
            unsafe { self.context.delete_shader(*shader) };
            self.shaders.remove(key);
        }
    }
}

impl Drop for ShaderManager {
    fn drop(&mut self) {
        for (_, shader) in &self.shaders {
            unsafe { self.context.delete_shader(*shader) };
        }
        self.shaders.clear();
    }
}

// -----------------------------------------------------------------------------
// Shader program
// -----------------------------------------------------------------------------

pub struct ShaderProgram {
    context: Arc<Context>,
    shader_manager: Arc<ShaderManager>,
    program: Program,
    linked: bool,
    shaders: Vec<Shader>,
    uniform_locations: BTreeMap<String, Option<UniformLocation>>,
}

impl ShaderProgram {
    pub fn new(context: Arc<Context>, shader_manager: Arc<ShaderManager>) -> Result<ShaderProgram, SimpleError> {
        let maybe_handle = unsafe { context.create_program() };
        if let Err(err) = maybe_handle {
            return Err(SimpleError::new(err));
        }

        Ok(ShaderProgram {
            context,
            shader_manager,
            program: maybe_handle.unwrap(),
            linked: false,
            shaders: vec![],
            uniform_locations: BTreeMap::new(),
        })
    }

    pub fn attach_shader<Q: ?Sized>(&mut self, key: &Q)
    where
        String : Borrow<Q> + Ord,
        Q: Ord,
    {
        let maybe_shader = self.shader_manager.get_shader(key);
        if let Some(shader) = maybe_shader {
            unsafe { self.context.attach_shader(self.program, *shader) };
            self.shaders.push(*shader);
        }
    }

    pub fn link(&mut self) -> Result<(), SimpleError> {
        if !self.linked {
            // Link program
            let link_succeed = unsafe {
                self.context.link_program(self.program);
                self.context.get_program_link_status(self.program)
            };
            if !link_succeed {
                return Err(SimpleError::new(unsafe {
                    self.context.get_program_info_log(self.program)
                }));
            }

            // Find and save uniform variables indexes
            self.uniform_locations.clear();
            unsafe {
                let unifoms_count = self.context.get_active_uniforms(self.program);
                for i in 0..unifoms_count {
                    let maybe_uniform = self.context.get_active_uniform(self.program, i);
                    if let Some(uniform) = maybe_uniform {
                        let name = uniform.name.clone();
                        self.uniform_locations.insert(
                            name,
                            self.context
                                .get_uniform_location(self.program, &uniform.name.clone()),
                        );
                    }
                }
            }

            self.linked = true;
        }

        Ok(())
    }

    pub fn use_program(&self) -> Result<(), SimpleError> {
        self.assert_linked()?;

        unsafe { self.context.use_program(Some(self.program)) };

        Ok(())
    }

    pub fn get_handle(&self) -> Program {
        self.program
    }

    pub fn is_linked(&self) -> bool {
        self.linked
    }

    pub fn bind_attrib_location(&self, index: u32, name: &str) {
        unsafe { self.context.bind_attrib_location(self.program, index, name) };
    }

    pub fn bind_frag_data_location(&self, color_number: u32, name: &str) {
        unsafe {
            self.context
                .bind_frag_data_location(self.program, color_number, name)
        };
    }

    pub fn set_uniform_value(&self, name: &str, value: GlslValue) {
        use glow::{FALSE as GL_FALSE, TRUE as GL_TRUE};
        use log::warn as log_warn;

        unsafe {
            // Get uniform value location index
            if !self.uniform_locations.contains_key(name) {
                log_warn!("Shader program has no uniform with name \"{}\"", name);
                return;
            }
            let location = self.uniform_locations[name];
            let location_ref = location.as_ref();

            match value {
                GlslValue::Float32(value) => self.context.uniform_1_f32(location_ref, value),
                GlslValue::Float32Vec2(value) => {
                    self.context.uniform_2_f32(location_ref, value.x, value.y)
                }
                GlslValue::Float32Vec3(value) => {
                    self.context
                        .uniform_3_f32(location_ref, value.x, value.y, value.z)
                }
                GlslValue::Float32Vec4(value) => {
                    self.context
                        .uniform_4_f32(location_ref, value.x, value.y, value.z, value.w)
                }
                GlslValue::Float64(value) => {
                    if self
                        .context
                        .supported_extensions()
                        .contains("ARB_gpu_shader_fp64")
                    {
                        log_warn!("Pass f64 uniforms is not supported yet (passing {})", value);
                    } else {
                        log_warn!(
                            "Your OpenGL version does not support f64 uniforms (passing {})",
                            value
                        );
                    }
                }
                GlslValue::Int32(value) => self.context.uniform_1_i32(location_ref, value),
                GlslValue::UnsignedInt32(value) => self.context.uniform_1_u32(location_ref, value),
                GlslValue::Bool(value) => self.context.uniform_1_u32(
                    location_ref,
                    if value {
                        GL_TRUE as u32
                    } else {
                        GL_FALSE as u32
                    },
                ),
                GlslValue::Float32Mat2(value) => {
                    self.context
                        .uniform_matrix_2_f32_slice(location_ref, false, value.as_slice())
                }
                GlslValue::Float32Mat3(value) => {
                    self.context
                        .uniform_matrix_3_f32_slice(location_ref, false, value.as_slice())
                }
                GlslValue::Float32Mat4(value) => {
                    self.context
                        .uniform_matrix_4_f32_slice(location_ref, false, value.as_slice())
                }
            }
        }
    }

    pub fn print_active_attribs(&self) {
        use log::{debug, warn};

        unsafe {
            let attribs_count = self.context.get_active_attributes(self.program);
            for i in 0..attribs_count {
                let maybe_attrib = self.context.get_active_attribute(self.program, i);
                if let Some(attrib) = maybe_attrib {
                    debug!(
                        "[ATTRIB] #{} {:?} {}",
                        i,
                        native_gl_value_type_to_keyword(attrib.atype),
                        attrib.name
                    );
                } else {
                    warn!("[ATTRIB] #{} NOT FOUND", i);
                }
            }
        }
    }

    pub fn print_active_uniforms(&self) {
        use log::{debug, warn};

        unsafe {
            let uniforms_count = self.context.get_active_uniforms(self.program);
            for i in 0..uniforms_count {
                let maybe_uniform = self.context.get_active_uniform(self.program, i);
                if let Some(uniform) = maybe_uniform {
                    debug!(
                        "[UNIFORM] #{} {:?} {}",
                        i,
                        native_gl_value_type_to_keyword(uniform.utype),
                        uniform.name
                    )
                } else {
                    warn!("[UNIFORM] #{} NOT FOUND", i);
                }
            }
        }
    }

    fn assert_linked(&self) -> Result<(), SimpleError> {
        if !self.linked {
            Err(SimpleError::new("Shader program not been linked"))
        } else {
            Ok(())
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            for shader in &self.shaders {
                self.context.detach_shader(self.program, *shader);
            }
            self.context.delete_program(self.program);
        }
    }
}

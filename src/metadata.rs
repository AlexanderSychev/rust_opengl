use glow::{Context, HasContext};
use semver::Version;
use std::collections::HashSet;
use std::sync::Arc;

static MIN_OPENGL_VERSION: Version = Version::new(4, 3, 0);

pub struct OpenGlMetadata {
    renderer: String,
    version_full: String,
    vendor: String,
    glsl_version: String,
    version: Version,
    extensions: HashSet<String>,
}

impl OpenGlMetadata {
    pub fn assert_version(&self) {
        if self.version < MIN_OPENGL_VERSION {
            panic!(
                "Application requies at least OpenGL v{}",
                MIN_OPENGL_VERSION
            );
        }
    }
}

impl std::fmt::Debug for OpenGlMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[OpenGL] Version: {}\n[OpenGL] Vendor: {}\n[OpenGL] Renderer: {}\n[OpenGL] GLSL Version: {}\n[OpenGL] {} extension(s) supported",
            self.version_full,
            self.vendor,
            self.renderer,
            self.glsl_version,
            self.extensions.len(),
        ))
    }
}

impl From<Arc<Context>> for OpenGlMetadata {
    fn from(gl: Arc<Context>) -> Self {
        let (
            renderer,
            version_full,
            vendor,
            glsl_version,
            extensions,
            major_version,
            minor_version,
        ) = unsafe {
            let extensions_count = gl.get_parameter_i32(glow::NUM_EXTENSIONS) as u32;
            let mut extensions = HashSet::with_capacity(extensions_count as usize);
            for i in 0..extensions_count {
                extensions.insert(gl.get_parameter_indexed_string(glow::EXTENSIONS, i));
            }

            (
                gl.get_parameter_string(glow::RENDERER),
                gl.get_parameter_string(glow::VERSION),
                gl.get_parameter_string(glow::VENDOR),
                gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION),
                extensions,
                gl.get_parameter_i32(glow::MAJOR_VERSION),
                gl.get_parameter_i32(glow::MINOR_VERSION),
            )
        };
        OpenGlMetadata {
            renderer,
            version_full,
            vendor,
            glsl_version,
            extensions,
            version: Version::new(major_version as u64, minor_version as u64, 0),
        }
    }
}

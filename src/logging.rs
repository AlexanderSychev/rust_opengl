pub fn gl_log_callback(source: u32, tp: u32, id: u32, severity: u32, message: &str) {
    use log::{debug, error, info, warn};

    let str_source = match source {
        glow::DEBUG_SOURCE_API => "[GL_DEBUG_SOURCE_API]",
        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "[GL_DEBUG_SOURCE_WINDOW_SYSTEM]",
        glow::DEBUG_SOURCE_THIRD_PARTY => "[GL_DEBUG_SOURCE_THIRD_PARTY]",
        glow::DEBUG_SOURCE_APPLICATION => "[GL_DEBUG_SOURCE_APPLICATION]",
        glow::DEBUG_SOURCE_OTHER => "[GL_DEBUG_SOURCE_OTHER]",
        _ => "[UNKNOWN]",
    };
    let str_type = match tp {
        glow::DEBUG_TYPE_ERROR => "[GL_DEBUG_TYPE_ERROR]",
        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "[GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR]",
        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "[GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR]",
        glow::DEBUG_TYPE_PORTABILITY => "[GL_DEBUG_TYPE_PORTABILITY]",
        glow::DEBUG_TYPE_PERFORMANCE => "[GL_DEBUG_TYPE_PERFORMANCE]",
        glow::DEBUG_TYPE_MARKER => "[GL_DEBUG_TYPE_MARKER]",
        glow::DEBUG_TYPE_PUSH_GROUP => "[GL_DEBUG_TYPE_PUSH_GROUP]",
        glow::DEBUG_TYPE_POP_GROUP => "[GL_DEBUG_TYPE_POP_GROUP]",
        glow::DEBUG_TYPE_OTHER => "[GL_DEBUG_TYPE_OTHER]",
        _ => "[UNKNOWN]",
    };

    match severity {
        glow::DEBUG_SEVERITY_HIGH => error!("#{} {} {} {}", id, str_source, str_type, message),
        glow::DEBUG_SEVERITY_MEDIUM => warn!("#{} {} {} {}", id, str_source, str_type, message),
        glow::DEBUG_SEVERITY_LOW => info!("#{} {} {} {}", id, str_source, str_type, message),
        _ => debug!("#{} {} {} {}", id, str_source, str_type, message),
    }
}

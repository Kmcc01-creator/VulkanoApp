use std::fmt;

#[derive(Debug)]
pub enum Error {
    GraphicsInitialization(String),
    WindowCreation(String),
    ResourceCreation(String),
    ResourceAccess(String),
    CommandRecordingError(String),
    OutOfMemory(String),
    RenderError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GraphicsInitialization(msg) => {
                write!(f, "Graphics initialization error: {}", msg)
            }
            Error::WindowCreation(msg) => write!(f, "Window creation error: {}", msg),
            Error::ResourceCreation(msg) => write!(f, "Resource creation error: {}", msg),
            Error::ResourceAccess(msg) => write!(f, "Resource access error: {}", msg),
            Error::CommandRecordingError(msg) => write!(f, "Command recording error: {}", msg),
            Error::OutOfMemory(msg) => write!(f, "Out of memory error: {}", msg),
            Error::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

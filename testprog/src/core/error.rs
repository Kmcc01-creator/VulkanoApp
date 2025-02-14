use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    WindowCreation(String),
    WindowEvent(String),
    GraphicsInitialization(String),
    RenderError(String),
    ResourceError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::WindowCreation(msg) => write!(f, "Window creation error: {}", msg),
            Error::WindowEvent(msg) => write!(f, "Window event error: {}", msg),
            Error::GraphicsInitialization(msg) => {
                write!(f, "Graphics initialization error: {}", msg)
            }
            Error::RenderError(msg) => write!(f, "Render error: {}", msg),
            Error::ResourceError(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl StdError for Error {}

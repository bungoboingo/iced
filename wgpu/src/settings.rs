//! Configure a renderer.
pub use crate::Antialiasing;

use crate::Font;

/// The settings of a [`Backend`].
///
/// [`Backend`]: crate::Backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The present mode of the [`Backend`].
    ///
    /// [`Backend`]: crate::Backend
    pub present_mode: wgpu::PresentMode,

    /// The internal graphics backend to use.
    pub internal_backend: wgpu::Backends,

    /// The default [`Font`] to use.
    pub default_font: Font,

    /// The default size of text.
    ///
    /// By default, it will be set to 20.
    pub default_text_size: u16,

    /// The antialiasing strategy that will be used for triangle primitives.
    ///
    /// By default, it is `None`.
    pub antialiasing: Option<Antialiasing>,
}

impl Settings {
    /// Creates new [`Settings`] using environment configuration.
    ///
    /// Specifically:
    ///
    /// - The `internal_backend` can be configured using the `WGPU_BACKEND`
    /// environment variable. If the variable is not set, the primary backend
    /// will be used. The following values are allowed:
    ///     - `vulkan`
    ///     - `metal`
    ///     - `dx12`
    ///     - `dx11`
    ///     - `gl`
    ///     - `webgpu`
    ///     - `primary`
    pub fn from_env() -> Self {
        Settings {
            internal_backend: backend_from_env()
                .unwrap_or(wgpu::Backends::all()),
            ..Self::default()
        }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            present_mode: wgpu::PresentMode::AutoVsync,
            internal_backend: wgpu::Backends::all(),
            default_font: Font::SansSerif,
            default_text_size: 20,
            antialiasing: None,
        }
    }
}

fn backend_from_env() -> Option<wgpu::Backends> {
    std::env::var("WGPU_BACKEND").ok().map(|backend| {
        match backend.to_lowercase().as_str() {
            "vulkan" => wgpu::Backends::VULKAN,
            "metal" => wgpu::Backends::METAL,
            "dx12" => wgpu::Backends::DX12,
            "dx11" => wgpu::Backends::DX11,
            "gl" => wgpu::Backends::GL,
            "webgpu" => wgpu::Backends::BROWSER_WEBGPU,
            "primary" => wgpu::Backends::PRIMARY,
            other => panic!("Unknown backend: {other}"),
        }
    })
}

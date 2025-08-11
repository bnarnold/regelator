pub mod admin;

use crate::extractors::Theme;
use charming::{Chart, ImageRenderer};
use color_eyre::eyre::{Context, Result};

/// Chart generation utilities for admin analytics
pub struct ChartGenerator;

impl ChartGenerator {
    /// Generate SVG chart from Chart configuration using server-side rendering
    pub fn generate_svg(chart: Chart, theme: Theme) -> Result<String> {
        let mut renderer = ImageRenderer::new(800, 400).theme(theme.into());
        renderer
            .render(&chart)
            .wrap_err("Failed to render chart to SVG")
    }

    /// Generate PNG chart from Chart configuration (future feature - requires ssr-raster)
    #[allow(dead_code)]
    pub fn generate_png(_chart: Chart) -> Result<Vec<u8>> {
        // TODO: Implement PNG rendering when ssr-raster feature is needed
        Err(color_eyre::eyre::eyre!("PNG rendering not implemented yet"))
    }
}

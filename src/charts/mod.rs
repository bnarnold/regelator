pub mod admin;

use charming::{Chart, ImageRenderer};
use color_eyre::eyre::{Context, Result};

/// Chart generation utilities for admin analytics
pub struct ChartGenerator;

impl ChartGenerator {
    /// Generate SVG chart from Chart configuration using server-side rendering
    pub fn generate_svg(chart: Chart) -> Result<String> {
        let mut renderer = ImageRenderer::new(800, 400);
        renderer.render(&chart)
            .wrap_err("Failed to render chart to SVG")
    }

    /// Generate PNG chart from Chart configuration (future feature - requires ssr-raster)
    #[allow(dead_code)]
    pub fn generate_png(_chart: Chart) -> Result<Vec<u8>> {
        // TODO: Implement PNG rendering when ssr-raster feature is needed
        Err(color_eyre::eyre::eyre!("PNG rendering not implemented yet"))
    }
}

/// Common chart styling and configuration
pub struct ChartTheme;

impl ChartTheme {
    /// Get base chart configuration with consistent styling
    pub fn base_config() -> Chart {
        Chart::new()
            .background_color("#f8f9fa") // Matches Pico CSS light theme
    }

    /// Color palette for charts (matching Pico CSS theme)
    pub fn color_palette() -> Vec<&'static str> {
        vec![
            "#007bff", // Primary blue
            "#28a745", // Success green
            "#ffc107", // Warning yellow
            "#dc3545", // Danger red
            "#6c757d", // Secondary gray
            "#17a2b8", // Info cyan
            "#fd7e14", // Orange
            "#6f42c1", // Purple
        ]
    }

    /// Colors for difficulty levels
    pub fn difficulty_colors() -> std::collections::HashMap<&'static str, &'static str> {
        let mut colors = std::collections::HashMap::new();
        colors.insert("beginner", "#28a745");    // Success green
        colors.insert("intermediate", "#ffc107"); // Warning yellow  
        colors.insert("advanced", "#dc3545");     // Danger red
        colors
    }
}
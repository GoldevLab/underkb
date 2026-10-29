pub mod admin;
pub mod comprimir_imagen_kb;
pub mod convertir_jpg_a_webp;
pub mod extraer_colores_imagen;
pub mod extension;
pub mod guide_background;
pub mod guide_compress;
pub mod guide_resize;
pub mod guide_webp;
pub mod guides;
pub mod index;
pub mod pricing;
pub mod privacy;
pub mod terms;
pub mod quitar_fondo;
pub mod redimensionar_imagen;

mod _registry;
pub use _registry::PagesRegistry;

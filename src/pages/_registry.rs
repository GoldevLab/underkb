use resuma::prelude::*;
use resuma::FlowPageRegistry;

pub struct PagesRegistry;

impl FlowPageRegistry for PagesRegistry {
    fn routes(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("/", "index"),
            ("/comprimir-imagen-kb", "comprimir_imagen_kb"),
            ("/convertir-jpg-a-webp", "convertir_jpg_a_webp"),
            ("/redimensionar-imagen", "redimensionar_imagen"),
            ("/quitar-fondo", "quitar_fondo"),
            ("/extraer-colores-imagen", "extraer_colores_imagen"),
        ]
    }

    fn layout_for(&self, pattern: &str) -> &'static [&'static str] {
        match pattern {
            _ => &["/"],
        }
    }

    fn render(&self, module: &str, req: FlowRequest) -> Option<View> {
        match module {
            "index" => Some(super::index::page(req)),
            "comprimir_imagen_kb" => Some(super::comprimir_imagen_kb::page(req)),
            "convertir_jpg_a_webp" => Some(super::convertir_jpg_a_webp::page(req)),
            "redimensionar_imagen" => Some(super::redimensionar_imagen::page(req)),
            "quitar_fondo" => Some(super::quitar_fondo::page(req)),
            "extraer_colores_imagen" => Some(super::extraer_colores_imagen::page(req)),
            _ => None,
        }
    }
}

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
            ("/privacy", "privacy"),
            ("/terms", "terms"),
            ("/pricing", "pricing"),
            ("/extension", "extension"),
            ("/guides", "guides"),
            ("/guides/compress-image-to-200kb", "guide_compress"),
            ("/guides/jpg-to-webp", "guide_webp"),
            ("/guides/resize-image", "guide_resize"),
            ("/guides/remove-white-background", "guide_background"),
            ("/admin", "admin"),
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
            "privacy" => Some(super::privacy::page(req)),
            "terms" => Some(super::terms::page(req)),
            "pricing" => Some(super::pricing::page(req)),
            "extension" => Some(super::extension::page(req)),
            "guides" => Some(super::guides::page(req)),
            "guide_compress" => Some(super::guide_compress::page(req)),
            "guide_webp" => Some(super::guide_webp::page(req)),
            "guide_resize" => Some(super::guide_resize::page(req)),
            "guide_background" => Some(super::guide_background::page(req)),
            "admin" => Some(super::admin::page(req)),
            _ => None,
        }
    }
}

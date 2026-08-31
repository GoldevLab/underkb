//! Spanish SEO landings that each run a real image tool.

use resuma::prelude::*;
use serde_json::json;

use crate::tool;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Compress,
    Convert,
    Resize,
    RemoveBg,
    Colors,
}

impl Tool {
    pub fn path(self) -> &'static str {
        match self {
            Self::Compress => "/comprimir-imagen-kb",
            Self::Convert => "/convertir-jpg-a-webp",
            Self::Resize => "/redimensionar-imagen",
            Self::RemoveBg => "/quitar-fondo",
            Self::Colors => "/extraer-colores-imagen",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Compress => "Comprimir imagen a KB — JPG, PNG y WebP | UnderKb",
            Self::Convert => "Convertir JPG a WebP online, gratis | UnderKb",
            Self::Resize => "Redimensionar imagen — ancho, alto y formato | UnderKb",
            Self::RemoveBg => "Quitar fondo de imagen (fondo plano) | UnderKb",
            Self::Colors => "Extraer colores de una imagen — paleta HEX | UnderKb",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Compress => {
                "Comprime una foto a 50, 200 o 500 KB. JPG, PNG o WebP. Sin cuenta. Sube hasta 20 MB."
            }
            Self::Convert => {
                "Pasa JPG o PNG a WebP con calidad ajustable. También PNG y JPG. Sin cuenta ni marca de agua."
            }
            Self::Resize => {
                "Cambia el ancho y el alto de una imagen. Encajar, recortar o estirar. JPG, WebP o PNG."
            }
            Self::RemoveBg => {
                "Quita un fondo liso (blanco o de estudio) y descarga PNG con transparencia. No es recorte de retrato por IA."
            }
            Self::Colors => {
                "Saca la paleta de colores de una foto: HEX y porcentaje. Copia los códigos para diseño o CSS."
            }
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::Compress => "Comprimir a un tamaño",
            Self::Convert => "Convertir formato",
            Self::Resize => "Redimensionar",
            Self::RemoveBg => "Quitar fondo",
            Self::Colors => "Paleta de colores",
        }
    }

    fn h1(self) -> &'static str {
        match self {
            Self::Compress => "Comprimir una imagen a KB",
            Self::Convert => "Convertir JPG a WebP",
            Self::Resize => "Redimensionar una imagen",
            Self::RemoveBg => "Quitar el fondo de una imagen",
            Self::Colors => "Extraer colores de una imagen",
        }
    }

    fn lead(self) -> &'static str {
        match self {
            Self::Compress => {
                "Elige un presupuesto en kilobytes. Bajamos calidad y, si hace falta, el tamaño en píxeles hasta entrar. Sin cuenta."
            }
            Self::Convert => {
                "WebP suele pesar menos que JPG en la web. Ajusta la calidad o exporta PNG si necesitas transparencia."
            }
            Self::Resize => {
                "Indica ancho, alto o ambos. Encajar respeta la proporción; rellenar recorta; estirar deforma."
            }
            Self::RemoveBg => {
                "Sirve para productos y capturas con fondo uniforme. Si el sujeto toca el borde, puede comerse un poco el contorno."
            }
            Self::Colors => {
                "Muestreamos la foto y agrupamos tonos parecidos. Obtienes HEX listos para copiar."
            }
        }
    }

    fn howto(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Compress => &[
                ("Sube el archivo", "JPG, PNG, WebP o GIF (un fotograma). Hasta 20 MB."),
                ("Pon el límite", "200 KB va bien para web y correo. 50 KB para miniaturas."),
                ("Descarga", "Primero bajamos calidad; si no cabe, reducimos el lado más largo."),
            ],
            Self::Convert => &[
                ("Sube un JPG o PNG", "También aceptamos WebP y GIF. HEIC lo convierte el navegador si puede."),
                ("Elige WebP y calidad", "80 es un buen equilibrio. 98 o más usa WebP sin pérdida."),
                ("Descarga el .webp", "Compáralo con el original: peso y dimensiones aparecen debajo."),
            ],
            Self::Resize => &[
                ("Sube la imagen", "Mismo límite de 20 MB que el resto de UnderKb."),
                ("Ancho y alto", "Con un solo valor mantenemos la proporción. Los dos juntos usan el modo que elijas."),
                ("Descarga", "JPG para fotos, PNG si hay transparencia, WebP si quieres un archivo más ligero."),
            ],
            Self::RemoveBg => &[
                ("Foto con fondo liso", "Blanco de estudio o un color uniforme funciona. Un paisaje no."),
                ("Ajusta la tolerancia", "Si queda halo, súbela. Si se come el objeto, bájala."),
                ("PNG transparente", "El preview usa un damero. Descarga y úsalo en la web o en un diseño."),
            ],
            Self::Colors => &[
                ("Sube la foto", "Logos, UI o fotos. Los píxeles muy transparentes se ignoran."),
                ("Cuántos colores", "Entre 3 y 12. Los tonos muy parecidos se fusionan."),
                ("Copia el HEX", "Cada muestra muestra el código y un porcentaje aproximado."),
            ],
        }
    }

    pub fn card_mark(self) -> &'static str {
        match self {
            Self::Compress => "kB",
            Self::Convert => "W",
            Self::Resize => "↔",
            Self::RemoveBg => "✂",
            Self::Colors => "#",
        }
    }

    pub fn card_title(self) -> &'static str {
        match self {
            Self::Compress => "Comprimir a KB",
            Self::Convert => "JPG a WebP",
            Self::Resize => "Redimensionar",
            Self::RemoveBg => "Quitar fondo",
            Self::Colors => "Extraer colores",
        }
    }

    pub fn card_blurb(self) -> &'static str {
        match self {
            Self::Compress => "Baja la foto a 50, 200 o 500 KB. Calidad primero, luego escala.",
            Self::Convert => "Pasa JPG o PNG a WebP (o al revés). Ajustas la calidad.",
            Self::Resize => "Cambia ancho y alto. Encajar, recortar o estirar.",
            Self::RemoveBg => "Fondo blanco o liso → PNG transparente. No es recorte de retrato.",
            Self::Colors => "Paleta HEX con un porcentaje aproximado. Copia los códigos.",
        }
    }

    pub fn card_cta(self) -> &'static str {
        "Abrir herramienta"
    }

    fn faq(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Compress => &[
                ("¿Puedo comprimir a 200 KB?", "Sí. 200 KB es el valor por defecto. También hay 50, 100, 500 y 1024."),
                ("¿Guardan mis fotos?", "El resultado vive en memoria unos 30 minutos para que lo descargues, y luego caduca."),
            ],
            Self::Convert => &[
                ("¿WebP es más pequeño que JPG?", "En fotos, casi siempre, con calidad 70–85. Gráficos planos a veces quedan mejor en PNG o WebP sin pérdida."),
                ("¿Se pierde transparencia?", "Si sales a JPG, sí (fondo blanco). WebP y PNG la conservan."),
            ],
            Self::Resize => &[
                ("¿Puedo poner solo el ancho?", "Sí. El alto se calcula para no deformar. Igual al revés."),
                ("¿Hay un máximo?", "El lado más largo en el servidor llega a 4096 px. Fotos enormes se reducen antes en el navegador."),
            ],
            Self::RemoveBg => &[
                ("¿Quita el fondo de un retrato?", "No es un modelo de personas. Si el fondo no es plano o el pelo toca el borde, el recorte será tosco."),
                ("¿Por qué sale PNG?", "Hace falta canal alfa. JPG no tiene transparencia."),
            ],
            Self::Colors => &[
                ("¿Son los colores exactos de cada píxel?", "No. Agrupamos tonos cercanos. Sirve para paletas, no para medición de imprenta."),
                ("¿Cuántos colores salen?", "Hasta los que pidas, si la foto los tiene. Una captura grisácea puede devolver menos."),
            ],
        }
    }
}

pub fn page(tool: Tool) -> View {
    let faq_ld = json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": tool.faq().iter().map(|(q, a)| json!({
            "@type": "Question",
            "name": q,
            "acceptedAnswer": { "@type": "Answer", "text": a }
        })).collect::<Vec<_>>()
    });
    let page_ld = json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "name": tool.title(),
        "description": tool.description(),
        "url": format!("https://underkb.fly.dev{}", tool.path())
    });

    let howto: Vec<View> = tool
        .howto()
        .iter()
        .map(|(h, p)| {
            view! {
                <li>
                    <h3>{*h}</h3>
                    <p>{*p}</p>
                </li>
            }
        })
        .collect();
    let faq: Vec<View> = tool
        .faq()
        .iter()
        .map(|(q, a)| {
            view! {
                <details>
                    <summary>{*q}</summary>
                    <p>{*a}</p>
                </details>
            }
        })
        .collect();

    let form = match tool {
        Tool::Compress => tool::compressor_es(),
        Tool::Convert => tool::converter(),
        Tool::Resize => tool::resizer(),
        Tool::RemoveBg => tool::remover(),
        Tool::Colors => tool::palette(),
    };

    let more = more_tools(tool);

    view! {
        <main class="home-page" lang="es">
            {View::raw(format!(
                r#"<script type="application/ld+json">{}</script><script type="application/ld+json">{}</script>"#,
                page_ld, faq_ld
            ))}
            <section class="hero">
                <p class="eyebrow">{tool.eyebrow()}</p>
                <h1>{tool.h1()}</h1>
                <p class="hero-lead">{tool.lead()}</p>
                {form}
            </section>
            {crate::ads::slot("landing-hero", "infeed")}
            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"Cómo usarlo"</h2>
                <ol class="howto-grid">{howto}</ol>
            </section>
            {crate::ads::slot("landing-mid", "infeed")}
            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"Preguntas"</h2>
                <div class="faq-list">{faq}</div>
            </section>
            {crate::ads::slot("landing-faq", "infeed")}
            <section class="features" aria-labelledby="more-title">
                <h2 id="more-title">"Otras herramientas"</h2>
                <ul class="feature-grid">{more}</ul>
            </section>
        </main>
    }
}

fn more_tools(current: Tool) -> Vec<View> {
    let all = [
        (Tool::Compress, "Comprimir a KB", "Límite real en kilobytes."),
        (Tool::Convert, "JPG → WebP", "Calidad ajustable."),
        (Tool::Resize, "Redimensionar", "Ancho, alto o recorte."),
        (Tool::RemoveBg, "Quitar fondo", "Fondos lisos a PNG."),
        (Tool::Colors, "Extraer colores", "Paleta HEX."),
    ];
    all.into_iter()
        .filter(|(t, _, _)| *t != current)
        .map(|(t, title, blurb)| {
            let href = t.path();
            view! {
                <li>
                    <h3>
                        <NavLink href={href}>{title}</NavLink>
                    </h3>
                    <p>{blurb}</p>
                </li>
            }
        })
        .collect()
}

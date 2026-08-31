use resuma::prelude::*;

use crate::landing::Tool;

pub fn page(_req: FlowRequest) -> View {
    let cards: Vec<View> = [
        Tool::Compress,
        Tool::Convert,
        Tool::Resize,
        Tool::RemoveBg,
        Tool::Colors,
    ]
    .into_iter()
    .map(tool_card)
    .collect();

    view! {
        <main class="home-page home-hub" lang="es">
            <section class="hero">
                <p class="eyebrow">"Herramientas de imagen"</p>
                <h1>"¿Qué quieres hacer con tu foto?"</h1>
                <p class="hero-lead">
                    "Elige una herramienta. Subes el archivo ahí, sin cuenta ni marca de agua. Hasta 20 MB."
                </p>
            </section>
            {crate::ads::slot("home-hero", "infeed")}
            <section class="tool-pick" aria-labelledby="tools-title">
                <h2 id="tools-title">"Herramientas"</h2>
                <div class="tool-grid">{cards}</div>
            </section>
            {crate::ads::slot("home-mid", "infeed")}
            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"Cómo va"</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>"Elige"</h3>
                        <p>"Comprimir a un tamaño, pasar a WebP, cambiar medidas, quitar un fondo liso o sacar colores."</p>
                    </li>
                    <li>
                        <h3>"Suelta la imagen"</h3>
                        <p>"JPG, PNG, WebP o GIF. En el teléfono también puedes elegir de la galería."</p>
                    </li>
                    <li>
                        <h3>"Descarga"</h3>
                        <p>"El archivo queda unos 30 minutos para bajarlo. No guardamos una galería tuya."</p>
                    </li>
                </ol>
            </section>
            {crate::ads::slot("home-faq", "infeed")}
            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"Preguntas"</h2>
                <div class="faq-list">
                    <details>
                        <summary>"¿Hay que registrarse?"</summary>
                        <p>"No. Entras, eliges la herramienta, subes y descargas."</p>
                    </details>
                    <details>
                        <summary>"¿Dónde comprimo a 200 KB?"</summary>
                        <p>"En Comprimir a KB. 200 KB es el valor por defecto."</p>
                    </details>
                    <details>
                        <summary>"¿Quitar fondo sirve para retratos?"</summary>
                        <p>"Solo fondos lisos (estudio, blanco). No es un recorte de personas por IA."</p>
                    </details>
                </div>
            </section>
        </main>
    }
}

fn tool_card(tool: Tool) -> View {
    let href = tool.path();
    let mark = tool.card_mark();
    let title = tool.card_title();
    let blurb = tool.card_blurb();
    let cta = tool.card_cta();
    view! {
        <NavLink href={href} class="tool-card">
            <span class="tool-card-mark" aria-hidden="true">{mark}</span>
            <h3 class="tool-card-title">{title}</h3>
            <p class="tool-card-blurb">{blurb}</p>
            <span class="tool-card-cta">{cta}</span>
        </NavLink>
    }
}

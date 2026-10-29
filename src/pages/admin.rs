use resuma::prelude::*;

use crate::stats::grouped;

pub fn page(req: FlowRequest) -> View {
    set_page_title("Admin | UnderKb");
    set_page_description("Private UnderKb statistics.");
    set_page_robots("noindex, nofollow");
    set_page_lang("es");
    if !crate::site::request_is_admin() {
        return login_view(req.query_param("e").is_some());
    }
    dash_view()
}

fn login_view(failed: bool) -> View {
    let notice = if failed {
        view! {
            <p class="hint admin-error" role="alert">
                "Ese usuario o clave no coincide."
            </p>
        }
    } else {
        view! { <span hidden=""></span> }
    };
    view! {
        <main class="content-section admin-page">
            <p class="eyebrow">"UnderKb"</p>
            <h1>"Admin"</h1>
            <p class="hero-lead">
                "Los trabajos terminados de la app están en esta página."
            </p>
            {notice}
            <Form submit={crate::actions::admin_login} class="hero-form admin-login">
                <label>
                    "Usuario"
                    <input name="user" type="text" autocomplete="username" required=true />
                </label>
                <label>
                    "Clave"
                    <input name="password" type="password" autocomplete="current-password" required=true />
                </label>
                <button type="submit" class="btn btn-primary">"Entrar"</button>
            </Form>
        </main>
    }
}

fn dash_view() -> View {
    let activity = crate::stats::snapshot();
    let compress = grouped(activity.compress);
    let convert = grouped(activity.convert);
    let resize = grouped(activity.resize);
    let background = grouped(activity.background);
    let colors = grouped(activity.colors);
    let store_note = if crate::stats::remote_ready() {
        "Guardado en la base de datos. Estos números no se publican."
    } else {
        "La base no respondió. Esta copia es de esta máquina y no se publica."
    };
    view! {
        <main class="content-section admin-page">
            <div class="admin-head">
                <div>
                    <p class="eyebrow">"UnderKb"</p>
                    <h1>"Estadísticas"</h1>
                </div>
                <Form submit={crate::actions::admin_logout}>
                    <input type="hidden" name="ok" value="1" />
                    <button type="submit" class="btn">"Cerrar sesión"</button>
                </Form>
            </div>
            <div class="admin-grid">
                {stat_card("Comprimir", compress)}
                {stat_card("Convertir", convert)}
                {stat_card("Redimensionar", resize)}
                {stat_card("Fondo", background)}
                {stat_card("Colores", colors)}
            </div>
            <p class="hint">{store_note}</p>
        </main>
    }
}

fn stat_card(label: &'static str, value: String) -> View {
    view! {
        <article class="admin-stat">
            <p class="admin-stat-value">{value}</p>
            <p class="hint">{label}</p>
        </article>
    }
}

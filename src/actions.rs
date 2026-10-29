use resuma::prelude::*;

#[data]
pub struct AdminLogin {
    user: String,
    password: String,
}

#[submit]
pub async fn admin_login(
    form: AdminLogin,
    req: &FlowRequest,
) -> std::result::Result<Redirect, SubmitError> {
    if !crate::admin_gate::allow_login(&crate::admin_gate::client_ip(req)) {
        return Err(SubmitError::new(
            "Demasiados intentos. Espera unos minutos.",
        ));
    }
    if !crate::admin_gate::admin_login_ok(&form.user, &form.password) {
        return Ok(redirect("/admin?e=1"));
    }
    Ok(redirect("/admin").with_cookie(crate::site::admin_cookie_header()))
}

#[data]
pub struct AdminLogout {
    ok: Option<String>,
}

#[submit]
pub async fn admin_logout(
    _form: AdminLogout,
    _req: &FlowRequest,
) -> std::result::Result<Redirect, SubmitError> {
    Ok(redirect("/admin").with_cookie(crate::site::admin_clear_cookie()))
}

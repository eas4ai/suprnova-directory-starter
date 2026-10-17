use super::listings::{actor_id, route_id};
use crate::{
    accounts::{
        self,
        queries::{self, Account, AuditEntry},
    },
    listings::{
        invalid,
        queries::{Page, Pagination},
    },
};
use suprnova::{InertiaProps, Request, Response, handler, inertia_response};
#[derive(InertiaProps)]
pub struct AccountsProps {
    pub accounts: Vec<Account>,
    pub q: String,
    pub pagination: Pagination,
}
#[derive(InertiaProps)]
pub struct AccountProps {
    pub account: Account,
}
#[derive(InertiaProps)]
pub struct AuditProps {
    pub entries: Vec<AuditEntry>,
    pub pagination: Pagination,
}
#[handler]
pub async fn index(req: Request) -> Response {
    let q = req.query_param("q").unwrap_or_default();
    let (accounts, pagination) = queries::search(
        actor_id().await?,
        &q,
        Page::parse(req.query_param("page"), req.query_param("per_page"))?,
    )
    .await?;
    inertia_response!(
        &req,
        "admin/Accounts",
        AccountsProps {
            accounts,
            q,
            pagination
        }
    )
}
#[handler]
pub async fn show(req: Request) -> Response {
    let account = queries::detail(actor_id().await?, route_id(&req)?).await?;
    inertia_response!(&req, "admin/AccountDetail", AccountProps { account })
}
#[handler]
pub async fn update(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    let input = req
        .json()
        .await
        .map_err(|_| invalid("account", "Check the account fields and try again."))?;
    accounts::save(actor, id, input).await?;
    if id == actor {
        let db = suprnova::DB::connection()?;
        if !accounts::is_active(actor).await?
            || !queries::has_permission_on(db.inner(), actor, accounts::MANAGE_PERMISSION).await?
            || !queries::has_permission_on(db.inner(), actor, "admin.access").await?
        {
            return suprnova::Redirect::to("/dashboard").into();
        }
    }
    suprnova::Redirect::to(format!("/admin/accounts/{id}")).into()
}
#[handler]
pub async fn audit(req: Request) -> Response {
    let (entries, pagination) = queries::audit(
        actor_id().await?,
        Page::parse(req.query_param("page"), req.query_param("per_page"))?,
    )
    .await?;
    inertia_response!(
        &req,
        "admin/Audit",
        AuditProps {
            entries,
            pagination
        }
    )
}

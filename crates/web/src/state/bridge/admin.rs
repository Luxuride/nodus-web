//! Moderation calls, and the one question that decides whether to show them.

use super::instance_transport;
use nodus_instance::actions::AdminActions;
use nodus_instance::model::{
    AdminAccount, DomainBlock, DomainSeverity, ModerationAction, Report, ReportState,
};

/// How many rows a moderation screen asks for.
///
/// Small on purpose: the queue is a work list a human reads, and a moderator who scrolls
/// past fifty reports has stopped reading them.
pub const PAGE: u16 = 20;

/// Whether this token may moderate.
///
/// Asked by trying, not by reading a role: `role` is a v1 field whose meaning changed
/// between versions, and a probe that fails is the same 403 the real screens would hit.
pub async fn is_moderator() -> bool {
    let Some(transport) = instance_transport() else {
        return false;
    };
    transport.reports(ReportState::Open, 1).await.is_ok()
}

/// Reports in the given state.
pub async fn reports(state: ReportState) -> Result<Vec<Report>, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .reports(state, PAGE)
        .await
        .map_err(|error| error.to_string())
}

/// Marks a report as handled.
pub async fn resolve_report(id: &str) -> Result<Report, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .resolve_report(id)
        .await
        .map_err(|error| error.to_string())
}

/// Reopens a handled report.
///
/// Unused until the queue grows a "resolved" tab; the SDK keeps it because a report
/// closed by mistake can only be reopened, never re-created.
#[expect(dead_code, reason = "the queue shows open reports only")]
pub async fn reopen_report(id: &str) -> Result<Report, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .reopen_report(id)
        .await
        .map_err(|error| error.to_string())
}

/// Accounts the instance knows about.
pub async fn accounts(local: bool) -> Result<Vec<AdminAccount>, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .admin_accounts(local, PAGE)
        .await
        .map_err(|error| error.to_string())
}

/// Applies a decision to one account.
pub async fn moderate(id: &str, action: ModerationAction) -> Result<AdminAccount, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .moderate_account(id, action)
        .await
        .map_err(|error| error.to_string())
}

/// Domains the instance refuses.
pub async fn domain_blocks() -> Result<Vec<DomainBlock>, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .domain_blocks(PAGE)
        .await
        .map_err(|error| error.to_string())
}

/// Refuses a domain.
pub async fn create_domain_block(
    domain: &str,
    severity: DomainSeverity,
) -> Result<DomainBlock, String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .create_domain_block(domain, severity)
        .await
        .map_err(|error| error.to_string())
}

/// Stops refusing a domain.
pub async fn delete_domain_block(id: &str) -> Result<(), String> {
    let transport = instance_transport().ok_or_else(|| "Not signed in.".to_owned())?;
    transport
        .delete_domain_block(id)
        .await
        .map_err(|error| error.to_string())
}

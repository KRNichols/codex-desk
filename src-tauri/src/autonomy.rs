//! Consequence ladder. Freedom inside boundaries — not identity-gate write HOLDs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Consequence {
    Read,
    Write,
    SendMergeDeploy,
    DeletePayPublish,
}

impl Consequence {
    pub fn as_str(self) -> &'static str {
        match self {
            Consequence::Read => "read",
            Consequence::Write => "write",
            Consequence::SendMergeDeploy => "send_merge_deploy",
            Consequence::DeletePayPublish => "delete_pay_publish",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Consequence::Read => "Read / research — automatic",
            Consequence::Write => "Write in workspace — automatic + checks",
            Consequence::SendMergeDeploy => "Send / merge / deploy — evidence + approval",
            Consequence::DeletePayPublish => "Delete / pay / publish — explicit human confirmation",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "send_merge_deploy" => Consequence::SendMergeDeploy,
            "delete_pay_publish" => Consequence::DeletePayPublish,
            "write" => Consequence::Write,
            _ => Consequence::Read,
        }
    }
}

fn negated(lower: &str, needle: &str) -> bool {
    let Some(idx) = lower.find(needle) else {
        return false;
    };
    let start = idx.saturating_sub(24);
    let window = &lower[start..idx];
    window.contains("do not")
        || window.contains("don't")
        || window.contains("never")
        || window.contains("without")
        || window.contains("forbid")
        || window.contains("not a")
}

fn has_intent(lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| lower.contains(n) && !negated(lower, n))
}

pub fn classify_text(text: &str) -> Consequence {
    let lower = text.to_ascii_lowercase();
    if has_intent(
        &lower,
        &[
            "npm publish",
            "cargo publish",
            "pypi publish",
            "wire payment",
            "send payment",
            "delete production",
            "drop table",
            "rm -rf",
            "git rm ",
        ],
    ) || (has_intent(&lower, &["publish "]) && has_intent(&lower, &["package", "crate", "release"]))
        || (lower.contains("pay ") && (lower.contains("invoice") || lower.contains("vendor")))
    {
        return Consequence::DeletePayPublish;
    }
    if has_intent(
        &lower,
        &[
            "git push",
            "git merge",
            "merge pull request",
            "merge the pr",
            "deploy to",
            "kubectl apply",
            "helm upgrade",
            "ship to prod",
            "send to production",
        ],
    ) {
        return Consequence::SendMergeDeploy;
    }
    if has_intent(
        &lower,
        &[
            "edit ",
            "write ",
            "patch ",
            "implement ",
            "fix ",
            "update ",
            "create file",
            "workspace-write",
            "hill-climb",
            "hill climb",
        ],
    ) {
        return Consequence::Write;
    }
    Consequence::Read
}

pub fn classify_goal(goal: &str, success_criteria: &str) -> Consequence {
    let a = classify_text(goal);
    let b = classify_text(success_criteria);
    unrank(rank(a).max(rank(b)))
}

fn rank(c: Consequence) -> u8 {
    match c {
        Consequence::Read => 0,
        Consequence::Write => 1,
        Consequence::SendMergeDeploy => 2,
        Consequence::DeletePayPublish => 3,
    }
}

fn unrank(n: u8) -> Consequence {
    match n {
        3 => Consequence::DeletePayPublish,
        2 => Consequence::SendMergeDeploy,
        1 => Consequence::Write,
        _ => Consequence::Read,
    }
}

/// Desk-performed action gate. Workspace writes are YOLO (no attestation).
pub fn gate(
    tier: Consequence,
    approved: bool,
    confirmed: bool,
    evidence: Option<&str>,
) -> Result<(), String> {
    match tier {
        Consequence::Read | Consequence::Write => Ok(()),
        Consequence::SendMergeDeploy => {
            if !approved {
                return Err(
                    "Send / merge / deploy needs evidence plus explicit approval before Desk performs it."
                        .into(),
                );
            }
            if evidence.map(|e| e.trim().len()).unwrap_or(0) < 8 {
                return Err(
                    "Send / merge / deploy approval requires evidence (what was verified, where)."
                        .into(),
                );
            }
            Ok(())
        }
        Consequence::DeletePayPublish => {
            if !confirmed {
                return Err(
                    "Delete / pay / publish needs explicit human confirmation before Desk performs it."
                        .into(),
                );
            }
            Ok(())
        }
    }
}

pub fn worker_violated_gate(worker: &str, approved: bool, confirmed: bool) -> Option<String> {
    let lower = worker.to_ascii_lowercase();
    if has_intent(
        &lower,
        &["git push", "git merge", "deployed to", "kubectl apply", "merged pull request"],
    ) && !approved
    {
        return Some(
            "HOLD: send/merge/deploy in the worker without evidence + approval. YOLO writes are not a send gate."
                .into(),
        );
    }
    if has_intent(
        &lower,
        &["npm publish", "cargo publish", "wired payment", "deleted production"],
    ) && !confirmed
    {
        return Some(
            "HOLD: delete/pay/publish in the worker without explicit human confirmation.".into(),
        );
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_default() {
        assert_eq!(classify_text("Summarize the README."), Consequence::Read);
    }

    #[test]
    fn write_is_yolo() {
        assert_eq!(
            classify_text("Fix the README smoke path and update docs."),
            Consequence::Write
        );
        assert!(gate(Consequence::Write, false, false, None).is_ok());
    }

    #[test]
    fn send_needs_evidence() {
        assert_eq!(
            classify_text("git push origin main and deploy to staging"),
            Consequence::SendMergeDeploy
        );
        assert!(gate(Consequence::SendMergeDeploy, false, false, None).is_err());
        assert!(gate(Consequence::SendMergeDeploy, true, false, Some("short")).is_err());
        assert!(gate(
            Consequence::SendMergeDeploy,
            true,
            false,
            Some("tests green on 03-desk-improver")
        )
        .is_ok());
    }

    #[test]
    fn delete_needs_confirm() {
        assert_eq!(
            classify_text("npm publish the package to the registry"),
            Consequence::DeletePayPublish
        );
        assert!(gate(Consequence::DeletePayPublish, true, false, Some("lots of evidence")).is_err());
        assert!(gate(Consequence::DeletePayPublish, true, true, None).is_ok());
    }

    #[test]
    fn negation_does_not_raise_tier() {
        assert_eq!(
            classify_text("Do not git push. Clarify the README without claiming ATO."),
            Consequence::Read
        );
    }

    #[test]
    fn worker_push_without_approval_holds() {
        assert!(worker_violated_gate("I ran git push origin main", false, false).is_some());
        assert!(worker_violated_gate("I ran git push origin main", true, false).is_none());
    }
}

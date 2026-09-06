//! Deterministic IL5 holds that do not depend on the model agreeing.

const ATO_CLAIMS: &[&str] = &[
    "we are authorized",
    "has an ato",
    "received an ato",
    "ato granted",
    "ato complete",
    "fedramp authorized",
    "fedramp authorization complete",
    "disa pa",
    "provisional authorization granted",
    "this product is authorized",
    "authorized to operate",
];

const WEAKEN: &[&str] = &[
    "remove encryption",
    "disable encryption",
    "plaintext sqlite",
    "store the pat in sqlite",
    "pat in sqlite",
    "drop audit",
    "delete audit",
    "remove audit",
    "skip hash chain",
    "allow http://",
    "cleartext endpoint",
    "phone home",
    "phone-home",
    "add telemetry",
];

pub fn claims_authorization(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    ATO_CLAIMS.iter().any(|p| lower.contains(p))
}

pub fn weakens_product_controls(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    WEAKEN.iter().any(|p| lower.contains(p))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Il5Row {
    pub owner: String,
    pub id: String,
    pub grade: String,
    pub evidence: String,
}

pub fn parse_il5_rows(text: &str) -> Vec<Il5Row> {
    let Some(start) = text.find("```il5-rows") else {
        return Vec::new();
    };
    let rest = &text[start + "```il5-rows".len()..];
    let Some(end) = rest.find("```") else {
        return Vec::new();
    };
    let body = &rest[..end];
    let mut rows = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('|').collect();
        if parts.len() < 4 {
            continue;
        }
        let owner = parts[0].trim();
        if owner != "product" && owner != "ao" {
            continue;
        }
        rows.push(Il5Row {
            owner: owner.to_string(),
            id: parts[1].trim().to_string(),
            grade: parts[2].trim().to_ascii_uppercase(),
            evidence: parts[3..].join("|").trim().to_string(),
        });
    }
    rows
}

pub fn product_rows_not_pass(rows: &[Il5Row]) -> Vec<&Il5Row> {
    rows.iter()
        .filter(|r| r.owner == "product" && r.grade != "PASS")
        .collect()
}

pub fn enforce_product_checklist(
    workspace: &std::path::Path,
    grade: &str,
    gaps: &str,
) -> (String, String) {
    let file = workspace.join("docs").join("il5").join("PRODUCT-CHECKLIST.md");
    if !file.is_file() {
        return (grade.to_string(), gaps.to_string());
    }
    let Ok(text) = std::fs::read_to_string(&file) else {
        return (
            "HOLD".into(),
            format!("HOLD: could not read {}.\n{gaps}", file.display()),
        );
    };
    let rows = parse_il5_rows(&text);
    let bad = product_rows_not_pass(&rows);
    if bad.is_empty() {
        return (grade.to_string(), gaps.to_string());
    }
    let extra = format!(
        "HOLD: product checklist rows not PASS: {}",
        bad.iter()
            .map(|r| format!("{}={}", r.id, r.grade))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let merged = if gaps.trim().is_empty() {
        extra
    } else {
        format!("{extra}\n{gaps}")
    };
    ("HOLD".into(), merged)
}

pub fn enforce_grade(worker: &str, grader_text: &str, grade: &str, gaps: &str) -> (String, String) {
    enforce_grade_with_autonomy(worker, grader_text, grade, gaps, false, false)
}

pub fn enforce_grade_with_autonomy(
    worker: &str,
    grader_text: &str,
    grade: &str,
    gaps: &str,
    approved: bool,
    confirmed: bool,
) -> (String, String) {
    let mut holds = Vec::new();
    if claims_authorization(worker) || claims_authorization(grader_text) {
        holds.push(
            "HOLD: text claims ATO / FedRAMP authorization / DISA PA. Desk never authorizes."
                .to_string(),
        );
    }
    if weakens_product_controls(worker) {
        holds.push(
            "HOLD: worker weakens encryption, audit, secret non-storage, TLS, or no-phone-home rules."
                .to_string(),
        );
    }
    if let Some(msg) = crate::autonomy::worker_violated_gate(worker, approved, confirmed) {
        holds.push(msg);
    }
    if holds.is_empty() {
        return (grade.to_string(), gaps.to_string());
    }
    let extra = holds.join("\n");
    let merged = if gaps.trim().is_empty() {
        extra
    } else {
        format!("{extra}\n{gaps}")
    };
    ("HOLD".into(), merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ato_claim_holds() {
        let (grade, gaps) = enforce_grade("This product is authorized to operate.", "", "PASS", "");
        assert_eq!(grade, "HOLD");
        assert!(gaps.contains("never authorizes"));
    }

    #[test]
    fn weaken_holds() {
        let (grade, _) = enforce_grade("We should remove encryption to pass.", "", "PASS", "ok");
        assert_eq!(grade, "HOLD");
    }

    #[test]
    fn clean_pass_survives() {
        let (grade, gaps) = enforce_grade("Updated README. No ATO claim.", "GRADE: PASS", "PASS", "none");
        assert_eq!(grade, "PASS");
        assert_eq!(gaps, "none");
    }

    #[test]
    fn push_without_approval_holds() {
        let (grade, gaps) = enforce_grade_with_autonomy(
            "I ran git push origin main after the docs edit.",
            "GRADE: PASS",
            "PASS",
            "none",
            false,
            false,
        );
        assert_eq!(grade, "HOLD");
        assert!(gaps.contains("send/merge/deploy"));
    }

    #[test]
    fn workspace_write_without_attestation_is_not_a_hold() {
        let (grade, gaps) = enforce_grade_with_autonomy(
            "Patched src-tauri/src/policy.rs and ran cargo test.",
            "GRADE: PASS",
            "PASS",
            "none",
            false,
            false,
        );
        assert_eq!(grade, "PASS");
        assert_eq!(gaps, "none");
    }

    #[test]
    fn checklist_holds_non_pass_product() {
        let text = "```il5-rows\nproduct|encrypted-store|PARTIAL|vault.rs\nao|fips-cmvp|MISSING|AO\n```\n";
        let rows = parse_il5_rows(text);
        assert_eq!(rows.len(), 2);
        assert_eq!(product_rows_not_pass(&rows).len(), 1);
    }

    #[test]
    fn checklist_pass_product_ignores_ao() {
        let text = "```il5-rows\nproduct|encrypted-store|PASS|vault.rs\nao|fips-cmvp|MISSING|AO\n```\n";
        assert!(product_rows_not_pass(&parse_il5_rows(text)).is_empty());
    }
}

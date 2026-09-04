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

pub fn enforce_grade(worker: &str, grader_text: &str, grade: &str, gaps: &str) -> (String, String) {
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
}

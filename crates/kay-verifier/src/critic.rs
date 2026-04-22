use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CriticVerdict {
    Pass,
    Fail,
}

#[derive(Debug, Clone)]
pub(crate) struct CriticResponse {
    pub verdict: CriticVerdict,
    pub reason: String,
}

impl CriticResponse {
    pub(crate) fn from_json(s: &str) -> Result<Self, String> {
        // TODO: implement in W-1 GREEN
        let _ = s;
        Err("not implemented".into())
    }

    pub(crate) fn is_pass(&self) -> bool {
        self.verdict == CriticVerdict::Pass
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CriticRole {
    TestEngineer,
    QAEngineer,
    EndUser,
}

impl CriticRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            CriticRole::TestEngineer => "test_engineer",
            CriticRole::QAEngineer => "qa_engineer",
            CriticRole::EndUser => "end_user",
        }
    }

    pub(crate) fn system_prompt(self) -> &'static str {
        // TODO: fill in W-2 GREEN
        ""
    }
}

pub(crate) struct CriticPrompt {
    pub role: CriticRole,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // --- CriticResponse parse tests (RED: from_json returns Err for all inputs) ---

    #[test]
    fn parse_pass_verdict() {
        let json = r#"{"verdict":"pass","reason":"all tests pass"}"#;
        let r = CriticResponse::from_json(json).expect("should parse pass");
        assert_eq!(r.verdict, CriticVerdict::Pass);
        assert_eq!(r.reason, "all tests pass");
    }

    #[test]
    fn parse_fail_verdict() {
        let json = r#"{"verdict":"fail","reason":"test X failed"}"#;
        let r = CriticResponse::from_json(json).expect("should parse fail");
        assert_eq!(r.verdict, CriticVerdict::Fail);
        assert_eq!(r.reason, "test X failed");
    }

    #[test]
    fn reject_unknown_verdict() {
        let json = r#"{"verdict":"maybe","reason":"not sure"}"#;
        assert!(CriticResponse::from_json(json).is_err(), "unknown verdict must be rejected");
    }

    #[test]
    fn reject_missing_verdict() {
        let json = r#"{"reason":"only reason, no verdict"}"#;
        assert!(CriticResponse::from_json(json).is_err(), "missing verdict must be rejected");
    }

    #[test]
    fn reject_missing_reason() {
        let json = r#"{"verdict":"pass"}"#;
        assert!(CriticResponse::from_json(json).is_err(), "missing reason must be rejected");
    }

    #[test]
    fn reject_additional_properties() {
        // ForgeCode schema hardening: deny_unknown_fields — extra fields must be rejected
        let json = r#"{"verdict":"pass","reason":"ok","extra":"ignored"}"#;
        assert!(CriticResponse::from_json(json).is_err(), "extra properties must be rejected");
    }

    #[test]
    fn is_pass_returns_true_for_pass() {
        let r = CriticResponse { verdict: CriticVerdict::Pass, reason: "ok".into() };
        assert!(r.is_pass());
    }

    #[test]
    fn is_pass_returns_false_for_fail() {
        let r = CriticResponse { verdict: CriticVerdict::Fail, reason: "bad".into() };
        assert!(!r.is_pass());
    }

    // --- CriticRole tests ---

    #[test]
    fn role_as_str_test_engineer() {
        assert_eq!(CriticRole::TestEngineer.as_str(), "test_engineer");
    }

    #[test]
    fn role_as_str_qa_engineer() {
        assert_eq!(CriticRole::QAEngineer.as_str(), "qa_engineer");
    }

    #[test]
    fn role_as_str_end_user() {
        assert_eq!(CriticRole::EndUser.as_str(), "end_user");
    }
}

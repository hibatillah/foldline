/// `Launch` is a free launch period and reads the `Paid` grants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Guest,
    Member,
    Trial,
    Paid,
    Launch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    Member,
    Trial,
    Paid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenFacts {
    pub plan: Plan,
    pub trial_ends_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchFacts {
    pub active: bool,
    pub ends_at: Option<i64>,
}

/// Debug builds only: forces the status shown and enforced (the grants stay built in).
pub const FORCED_STATUS_VARIABLE: &str = "NOTCH_ACCOUNT_STATUS";

/// `guest`, `member`, `trial`, `paid` or `launch` in any case and with spaces around; anything else is `None`.
pub fn parse_forced_status(text: Option<&str>) -> Option<Status> {
    match text?.trim().to_ascii_lowercase().as_str() {
        "guest" => Some(Status::Guest),
        "member" => Some(Status::Member),
        "trial" => Some(Status::Trial),
        "paid" => Some(Status::Paid),
        "launch" => Some(Status::Launch),
        _ => None,
    }
}

/// The status `NOTCH_ACCOUNT_STATUS` forces in a debug build; a release build never reads it.
pub fn forced_status() -> Option<Status> {
    if !cfg!(debug_assertions) {
        return None;
    }
    parse_forced_status(std::env::var(FORCED_STATUS_VARIABLE).ok().as_deref())
}

/// The first matching rule wins: no token, paid, running trial, running launch, member.
pub fn effective_status(token: Option<&TokenFacts>, launch: Option<&LaunchFacts>, now_unix: i64) -> Status {
    let Some(token) = token else {
        return Status::Guest;
    };
    if token.plan == Plan::Paid {
        return Status::Paid;
    }
    // A trial without an end time never counts as running.
    if token.plan == Plan::Trial && token.trial_ends_at.is_some_and(|ends_at| ends_at > now_unix) {
        return Status::Trial;
    }
    let launch_running =
        launch.is_some_and(|facts| facts.active && facts.ends_at.is_none_or(|ends_at| ends_at > now_unix));
    if launch_running {
        return Status::Launch;
    }
    Status::Member
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_000_000;

    fn token(plan: Plan, trial_ends_at: Option<i64>) -> TokenFacts {
        TokenFacts { plan, trial_ends_at }
    }

    fn launch(active: bool, ends_at: Option<i64>) -> LaunchFacts {
        LaunchFacts { active, ends_at }
    }

    #[test]
    fn every_status_rule_picks_the_first_match() {
        assert_eq!(effective_status(None, None, NOW), Status::Guest);
        let ended_launch = launch(true, Some(NOW - 1));
        assert_eq!(effective_status(Some(&token(Plan::Paid, None)), Some(&ended_launch), NOW), Status::Paid);
        assert_eq!(effective_status(Some(&token(Plan::Trial, Some(NOW + 1))), None, NOW), Status::Trial);
        assert_eq!(
            effective_status(Some(&token(Plan::Trial, Some(NOW - 1))), Some(&launch(false, None)), NOW),
            Status::Member
        );
        assert_eq!(effective_status(Some(&token(Plan::Member, None)), Some(&launch(true, None)), NOW), Status::Launch);
        assert_eq!(effective_status(Some(&token(Plan::Member, None)), Some(&ended_launch), NOW), Status::Member);
        assert_eq!(effective_status(None, Some(&launch(true, None)), NOW), Status::Guest);
    }

    #[test]
    fn a_forced_status_is_read_in_any_case_and_anything_else_is_ignored() {
        assert_eq!(parse_forced_status(Some(" Paid ")), Some(Status::Paid));
        assert_eq!(parse_forced_status(Some("guest")), Some(Status::Guest));
        assert_eq!(parse_forced_status(Some("LAUNCH")), Some(Status::Launch));
        assert_eq!(parse_forced_status(Some("premium")), None);
        assert_eq!(parse_forced_status(None), None);
    }

    #[test]
    fn an_ended_trial_falls_through_to_a_running_launch() {
        assert_eq!(
            effective_status(Some(&token(Plan::Trial, Some(NOW - 1))), Some(&launch(true, Some(NOW + 1))), NOW),
            Status::Launch
        );
    }
}

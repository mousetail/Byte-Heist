use std::fmt::Display;

use crate::slug::Slug;

pub enum ChallengePage<'a> {
    Solve { language: Option<&'a str> },
    View,
    Solutions { language: Option<&'a str> },
    Edit,
}

pub struct ChallengeUrl<'a> {
    challenge_id: i32,
    challenge_name: Option<&'a str>,
    page: ChallengePage<'a>,
}

impl Display for ChallengeUrl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/challenge/{}", self.challenge_id)?;
        match self.challenge_name {
            Some(e) => write!(f, "{}", Slug(e))?,
            None => write!(f, "-")?,
        };
        match self.page {
            ChallengePage::Solve { language } => write!(f, "/solve/{}", language.unwrap_or("")),
            ChallengePage::View => write!(f, "/view"),
            ChallengePage::Solutions { language } => {
                write!(f, "/solutions/{}", language.unwrap_or(""))
            }
            ChallengePage::Edit => write!(f, "/edit"),
        }?;

        Ok(())
    }
}

pub fn get_url_for_challenge<'a>(
    challenge_id: i32,
    challenge_name: Option<&'a str>,
    page: ChallengePage<'a>,
) -> ChallengeUrl<'a> {
    ChallengeUrl {
        challenge_id,
        challenge_name,
        page,
    }
}

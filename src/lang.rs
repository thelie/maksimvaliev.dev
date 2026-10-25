//! Site languages. Shared by `build.rs` (rendering) and the server (negotiation),
//! so each side uses only part of it.
#![allow(dead_code)]

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ru,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Ru];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Lang::En => "/en",
            Lang::Ru => "/ru",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Ru => "RU",
        }
    }

    pub fn locale(self) -> &'static str {
        match self {
            Lang::En => "en_US",
            Lang::Ru => "ru_RU",
        }
    }

    /// Picks the best supported language from an `Accept-Language` header, defaulting to English.
    pub fn negotiate(header: &str) -> Lang {
        header
            .split(',')
            .filter_map(|part| {
                let mut params = part.split(';');
                let primary = params.next()?.trim().split('-').next()?;
                let q = match params.find_map(|p| p.trim().strip_prefix("q=")) {
                    Some(q) => q.parse::<f32>().ok()?,
                    None => 1.0,
                };
                let lang = Lang::ALL.into_iter().find(|l| primary.eq_ignore_ascii_case(l.code()))?;
                (q > 0.0).then_some((lang, q))
            })
            // Keep the earliest entry on ties, as listed order expresses preference too.
            .fold(None, |best: Option<(Lang, f32)>, cur| match best {
                Some(b) if b.1 >= cur.1 => Some(b),
                _ => Some(cur),
            })
            .map_or(Lang::En, |(lang, _)| lang)
    }
}

#[cfg(test)]
mod tests {
    use super::Lang;

    #[test]
    fn negotiates_language() {
        let cases = [
            ("", Lang::En),
            ("ru-RU,ru;q=0.9,en-US;q=0.8", Lang::Ru),
            ("en-US,en;q=0.9,ru;q=0.8", Lang::En),
            ("de-DE,ru;q=0.5", Lang::Ru),
            ("ru;q=0.4,en;q=0.6", Lang::En),
            ("ru;q=0,de", Lang::En),
            ("RU", Lang::Ru),
        ];
        for (header, want) in cases {
            assert!(Lang::negotiate(header) == want, "{header:?}");
        }
    }
}

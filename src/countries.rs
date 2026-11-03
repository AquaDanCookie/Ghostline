const COUNTRIES: &[(&str, &str)] = &[
    ("DE", "Germany"),
    ("PL", "Poland"),
    ("GB", "United Kingdom"),
    ("CR", "Costa Rica"),
    ("LU", "Luxembourg"),
    ("NO", "Norway"),
    ("US", "United States"),
    ("IL", "Israel"),
    ("IT", "Italy"),
    ("DK", "Denmark"),
    ("DZ", "Algeria"),
    ("AR", "Argentina"),
    ("NL", "Netherlands"),
    ("AU", "Australia"),
    ("BG", "Bulgaria"),
    ("FR", "France"),
    ("VN", "Vietnam"),
    ("JP", "Japan"),
    ("GR", "Greece"),
    ("RS", "Serbia"),
    ("RO", "Romania"),
    ("ES", "Spain"),
    ("MT", "Malta"),
    ("PH", "Philippines"),
    ("PA", "Panama"),
    ("CA", "Canada"),
    ("BE", "Belgium"),
    ("MD", "Moldova"),
    ("AT", "Austria"),
    ("IE", "Ireland"),
    ("MC", "Monaco"),
    ("CH", "Switzerland"),
    ("UA", "Ukraine"),
    ("MA", "Morocco"),
    ("SE", "Sweden"),
    ("ZA", "South Africa"),
    ("QA", "Qatar"),
    ("AD", "Andorra"),
    ("BA", "Bosnia and Herzegovina"),
    ("KH", "Cambodia"),
    ("HK", "Hong Kong"),
    ("LT", "Lithuania"),
    ("CZ", "Czechia"),
    ("GL", "Greenland"),
    ("SG", "Singapore"),
    ("IM", "Isle of Man"),
    ("TR", "Turkey"),
    ("GE", "Georgia"),
    ("MO", "Macao"),
    ("RU", "Russia"),
    ("EG", "Egypt"),
    ("VE", "Venezuela"),
    ("BR", "Brazil"),
    ("FI", "Finland"),
    ("CY", "Cyprus"),
    ("LV", "Latvia"),
    ("IN", "India"),
    ("HR", "Croatia"),
    ("AL", "Albania"),
    ("TH", "Thailand"),
    ("IR", "Iran"),
    ("MK", "North Macedonia"),
    ("SK", "Slovakia"),
    ("BY", "Belarus"),
    ("IS", "Iceland"),
    ("CO", "Colombia"),
    ("KE", "Kenya"),
    ("KR", "South Korea"),
    ("PK", "Pakistan"),
    ("PT", "Portugal"),
    ("TW", "Taiwan"),
    ("BS", "Bahamas"),
    ("ID", "Indonesia"),
    ("LI", "Liechtenstein"),
    ("MN", "Mongolia"),
    ("MY", "Malaysia"),
    ("AE", "United Arab Emirates"),
    ("ME", "Montenegro"),
    ("SA", "Saudi Arabia"),
    ("CN", "China"),
    ("MX", "Mexico"),
    ("HU", "Hungary"),
    ("AM", "Armenia"),
    ("EE", "Estonia"),
    ("LK", "Sri Lanka"),
    ("BD", "Bangladesh"),
    ("CL", "Chile"),
    ("NG", "Nigeria"),
    ("SI", "Slovenia"),
    ("KZ", "Kazakhstan"),
    ("NZ", "New Zealand"),
    ("BO", "Bolivia"),
    ("DO", "Dominican Republic"),
    ("EC", "Ecuador"),
    ("GT", "Guatemala"),
    ("LA", "Laos"),
    ("MM", "Myanmar"),
    ("NP", "Nepal"),
    ("PE", "Peru"),
    ("UY", "Uruguay"),
];

const ALIASES: &[(&str, &str)] = &[
    ("uk", "GB"),
    ("u.k.", "GB"),
    ("great britain", "GB"),
    ("britain", "GB"),
    ("england", "GB"),
    ("usa", "US"),
    ("u.s.a.", "US"),
    ("america", "US"),
    ("united states of america", "US"),
    ("uae", "AE"),
    ("emirates", "AE"),
    ("holland", "NL"),
    ("czech republic", "CZ"),
    ("south korea", "KR"),
    ("korea", "KR"),
    ("russia", "RU"),
    ("vietnam", "VN"),
    ("turkey", "TR"),
    ("macau", "MO"),
    ("macedonia", "MK"),
    ("bosnia & herzegovina", "BA"),
    ("burma", "MM"),
];

pub fn name_for(code: &str) -> Option<&'static str> {
    COUNTRIES
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(code))
        .map(|(_, name)| *name)
}

pub fn all() -> &'static [(&'static str, &'static str)] {
    COUNTRIES
}

pub fn find_in_text(text: &str) -> Option<(&'static str, &'static str)> {
    let lower = text.to_lowercase();

    // Aliases first: official names such as "Macau SAR China" contain another
    // country name and must resolve to the more specific territory.
    for (alias, code) in ALIASES {
        if contains_phrase(&lower, alias) {
            if let Some(name) = name_for(code) {
                return Some((code, name));
            }
        }
    }
    for (code, name) in COUNTRIES {
        if contains_phrase(&lower, &name.to_lowercase()) {
            return Some((code, name));
        }
    }

    for token in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        if token.len() == 2 {
            if let Some(name) = name_for(token) {
                return Some((
                    COUNTRIES
                        .iter()
                        .find(|(code, _)| code.eq_ignore_ascii_case(token))
                        .map(|(code, _)| *code)
                        .unwrap_or(""),
                    name,
                ));
            }
        }
    }
    None
}

fn contains_phrase(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    let mut start = 0;
    while let Some(offset) = haystack[start..].find(needle) {
        let index = start + offset;
        let before_ok = index == 0
            || !haystack[..index]
                .chars()
                .next_back()
                .map(|c| c.is_ascii_alphanumeric())
                .unwrap_or(false);
        let end = index + needle.len();
        let after_ok = end == haystack.len()
            || !haystack[end..]
                .chars()
                .next()
                .map(|c| c.is_ascii_alphanumeric())
                .unwrap_or(false);
        if before_ok && after_ok {
            return true;
        }
        start = end;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_country_names_and_codes() {
        assert_eq!(find_in_text("CG_DE_Berlin_01").map(|v| v.0), Some("DE"));
        assert_eq!(
            find_in_text("CyberGhost Germany Frankfurt").map(|v| v.1),
            Some("Germany")
        );
        assert_eq!(find_in_text("us-new-york-01").map(|v| v.0), Some("US"));
        assert_eq!(find_in_text("netherlands").map(|v| v.0), Some("NL"));
        assert_eq!(find_in_text("holland").map(|v| v.0), Some("NL"));
        assert_eq!(
            find_in_text("CyberGhost Bosnia & Herzegovina").map(|v| v.0),
            Some("BA")
        );
        assert_eq!(find_in_text("myanmar").map(|v| v.0), Some("MM"));
        assert_eq!(find_in_text("Macau SAR China").map(|v| v.0), Some("MO"));
        assert!(find_in_text("profile-01").is_none());
    }

    #[test]
    fn does_not_match_inside_words() {
        assert!(find_in_text("indiana").is_none());
        assert!(find_in_text("germanyville").is_none());
    }
}

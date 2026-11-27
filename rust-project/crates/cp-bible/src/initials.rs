/// Chinese book initials in canonical order, starting with Genesis.
pub const CHINESE_BOOK_INITIALS: [&str; 66] = [
    "csj", "caj", "lwj", "msj", "smj", "ysyj", "ssj", "ldj", "smejs", "smejx", "lwjs", "lwjx",
    "ldzs", "ldzx", "yslj", "nxmj", "ystj", "ybj", "sp", "zy", "cds", "yg", "ysys", "ylms", "ylag",
    "yxjs", "dyls", "hxs", "yes", "ams", "ebdys", "yns", "mjs", "nhs", "hbgs", "xfys", "hgs",
    "sjlys", "mljs", "mtfy", "mkfy", "ljfy", "yhfy", "stxz", "lms", "gldqs", "gldhs", "jlts",
    "yfss", "flbs", "glxs", "tsljqs", "tsljhs", "tmtqs", "tmths", "tds", "flms", "xbls", "ygs",
    "bdqs", "bdhs", "yhys", "yhes", "yhss", "yds", "qsl",
];

/// Return one-based book IDs ranked by exact, prefix, then ordered subsequence match.
pub fn candidates(token: &str) -> Vec<i32> {
    let query = token.trim().to_ascii_lowercase();
    if query.len() < 2 || !query.bytes().all(|c| c.is_ascii_lowercase()) {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for (index, &initials) in CHINESE_BOOK_INITIALS.iter().enumerate() {
        let mut letters = initials.bytes();
        let rank = if initials == query {
            0
        } else if initials.starts_with(&query) {
            1
        } else if query.bytes().all(|q| letters.any(|c| c == q)) {
            2
        } else {
            continue;
        };
        matches.push((rank, initials.len(), index as i32 + 1));
    }
    matches.sort_unstable();
    matches.into_iter().map(|(_, _, id)| id).collect()
}

pub fn unique_quick_book_id(token: &str) -> Option<i32> {
    match candidates(token).as_slice() {
        [id] => Some(*id),
        _ => None,
    }
}

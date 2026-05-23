use super::ast::{AssertTarget, BodyTarget, PathSegment};

pub(super) fn parse_target(tok: &str) -> Result<AssertTarget, String> {
    match tok {
        "status" => return Ok(AssertTarget::Status),
        "duration" => return Ok(AssertTarget::Duration),
        "body" => return Ok(AssertTarget::Body(BodyTarget::Raw)),
        _ => {}
    }

    if let Some(rest) = tok.strip_prefix("body.") {
        return Ok(AssertTarget::Body(BodyTarget::Path(parse_path(rest))));
    }
    // body[0] or body[0].foo
    if let Some(rest) = tok.strip_prefix("body") {
        if rest.starts_with('[') {
            return Ok(AssertTarget::Body(BodyTarget::Path(parse_path(rest))));
        }
    }

    if let Some(rest) = tok.strip_prefix("header.") {
        return Ok(AssertTarget::Header(rest.to_lowercase()));
    }

    if let Some(rest) = tok.strip_prefix("cookie.") {
        let (name, attr) = match rest.split_once('.') {
            Some((name, attr)) => (name.to_string(), Some(attr.to_lowercase())),
            None => (rest.to_string(), None),
        };
        return Ok(AssertTarget::Cookie(name, attr));
    }

    Err(format!("unknown assert target: {tok:?}"))
}

/// Parse a dot-separated path, handling bracket index notation.
/// e.g. `"user.name"` → `[Key("user"), Key("name")]`
///      `"items[0]"` → `[Key("items"), Index(0)]`
///      `"[0].id"` → `[Index(0), Key("id")]`
pub(super) fn parse_path(path: &str) -> Vec<PathSegment> {
    // Leading bracket: [0].foo
    if path.starts_with('[') {
        if let Some(close) = path.find(']') {
            let idx_str = &path[1..close];
            let rest = &path[close + 1..];
            let rest = rest.strip_prefix('.').unwrap_or(rest);
            let mut segs = Vec::new();
            if let Ok(idx) = idx_str.parse::<usize>() {
                segs.push(PathSegment::Index(idx));
            }
            if !rest.is_empty() {
                segs.extend(parse_path(rest));
            }
            return segs;
        }
    }

    let mut segments = Vec::new();
    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Some((key, rest)) = part.split_once('[') {
            if !key.is_empty() {
                segments.push(PathSegment::Key(key.to_string()));
            }
            if let Some(idx_str) = rest.strip_suffix(']') {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    segments.push(PathSegment::Index(idx));
                }
            }
        } else {
            segments.push(PathSegment::Key(part.to_string()));
        }
    }
    segments
}

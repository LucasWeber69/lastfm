/// Email normalization to prevent duplicate account abuse
/// Inspired by Duolicious anti-abuse measures

pub fn normalize_email(email: &str) -> String {
    let email = email.trim().to_lowercase();
    
    // Split into local and domain parts
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return email;
    }
    
    let (local, domain) = (parts[0], parts[1]);
    
    // Handle Gmail-specific normalization
    // gmail.com and googlemail.com are the same
    // Remove dots and plus addressing
    let normalized_domain = if domain == "googlemail.com" {
        "gmail.com"
    } else {
        domain
    };
    
    let normalized_local = if normalized_domain == "gmail.com" {
        // Remove dots and anything after +
        local.replace('.', "")
            .split('+')
            .next()
            .unwrap_or(local)
            .to_string()
    } else {
        // For other domains, just remove plus addressing
        local.split('+')
            .next()
            .unwrap_or(local)
            .to_string()
    };
    
    format!("{}@{}", normalized_local, normalized_domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_email() {
        assert_eq!(
            normalize_email("john.doe+tag@gmail.com"),
            "johndoe@gmail.com"
        );
        assert_eq!(
            normalize_email("John.Doe@Gmail.com"),
            "johndoe@gmail.com"
        );
        assert_eq!(
            normalize_email("user@googlemail.com"),
            "user@gmail.com"
        );
        assert_eq!(
            normalize_email("user+tag@example.com"),
            "user@example.com"
        );
    }
}

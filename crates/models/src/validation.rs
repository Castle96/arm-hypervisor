use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Validates container name constraints
/// - Must be lowercase alphanumeric with hyphens or dots
/// - Must start with alphanumeric
/// - Must be 1-64 characters
pub fn validate_container_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::new(
            "name",
            "Container name cannot be empty",
        ));
    }

    if name.len() > 64 {
        return Err(ValidationError::new(
            "name",
            "Container name must be 64 characters or fewer",
        ));
    }

    if !name.chars().next().unwrap().is_ascii_lowercase() {
        return Err(ValidationError::new(
            "name",
            "Container name must start with lowercase alphanumeric",
        ));
    }

    for ch in name.chars() {
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' && ch != '.' {
            return Err(ValidationError::new(
                "name",
                "Container name can only contain lowercase alphanumeric, hyphens, and dots",
            ));
        }
    }

    Ok(())
}

/// Validates CPU limit
/// - Must be between 1 and 128 cores
pub fn validate_cpu_limit(limit: u32) -> Result<(), ValidationError> {
    if limit < 1 || limit > 128 {
        return Err(ValidationError::new(
            "config.cpu_limit",
            "CPU limit must be between 1 and 128",
        ));
    }
    Ok(())
}

/// Validates memory limit in bytes
/// - Must be at least 64MB
/// - Must not exceed 1TB
pub fn validate_memory_limit(limit: u64) -> Result<(), ValidationError> {
    let min_memory = 64 * 1024 * 1024; // 64MB
    let max_memory = 1024u64 * 1024 * 1024 * 1024; // 1TB

    if limit < min_memory {
        return Err(ValidationError::new(
            "config.memory_limit",
            "Memory limit must be at least 64MB",
        ));
    }

    if limit > max_memory {
        return Err(ValidationError::new(
            "config.memory_limit",
            "Memory limit must not exceed 1TB",
        ));
    }

    Ok(())
}

/// Validates disk limit in bytes
/// - Must be at least 100MB
/// - Must not exceed 10TB
pub fn validate_disk_limit(limit: u64) -> Result<(), ValidationError> {
    let min_disk = 100 * 1024 * 1024; // 100MB
    let max_disk = 10u64 * 1024 * 1024 * 1024 * 1024; // 10TB

    if limit < min_disk {
        return Err(ValidationError::new(
            "config.disk_limit",
            "Disk limit must be at least 100MB",
        ));
    }

    if limit > max_disk {
        return Err(ValidationError::new(
            "config.disk_limit",
            "Disk limit must not exceed 10TB",
        ));
    }

    Ok(())
}

/// Validates template name
/// - Must be lowercase alphanumeric
pub fn validate_template(template: &str) -> Result<(), ValidationError> {
    if template.is_empty() {
        return Err(ValidationError::new("template", "Template cannot be empty"));
    }

    if template.len() > 32 {
        return Err(ValidationError::new(
            "template",
            "Template name must be 32 characters or fewer",
        ));
    }

    if !template
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ValidationError::new(
            "template",
            "Template can only contain lowercase alphanumeric and hyphens",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_container_name() {
        // Valid names
        assert!(validate_container_name("test").is_ok());
        assert!(validate_container_name("test-container").is_ok());
        assert!(validate_container_name("web-server-1").is_ok());
        assert!(validate_container_name("a").is_ok());

        // Invalid names
        assert!(validate_container_name("").is_err());
        assert!(validate_container_name("Test").is_err());
        assert!(validate_container_name("test_server").is_err());
        assert!(validate_container_name("test server").is_err());
        assert!(validate_container_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_cpu_limit() {
        assert!(validate_cpu_limit(1).is_ok());
        assert!(validate_cpu_limit(4).is_ok());
        assert!(validate_cpu_limit(128).is_ok());

        assert!(validate_cpu_limit(0).is_err());
        assert!(validate_cpu_limit(129).is_err());
    }

    #[test]
    fn test_validate_memory_limit() {
        let min_ok = 64 * 1024 * 1024; // 64MB
        let max_ok = 512 * 1024 * 1024 * 1024; // 512GB

        assert!(validate_memory_limit(min_ok).is_ok());
        assert!(validate_memory_limit(max_ok).is_ok());

        assert!(validate_memory_limit(min_ok - 1).is_err());
        assert!(validate_memory_limit(1024u64 * 1024 * 1024 * 1024 + 1).is_err());
    }

    #[test]
    fn test_validate_disk_limit() {
        let min_ok = 100 * 1024 * 1024; // 100MB
        let max_ok = 5u64 * 1024 * 1024 * 1024 * 1024; // 5TB

        assert!(validate_disk_limit(min_ok).is_ok());
        assert!(validate_disk_limit(max_ok).is_ok());

        assert!(validate_disk_limit(min_ok - 1).is_err());
        assert!(validate_disk_limit(10u64 * 1024 * 1024 * 1024 * 1024 + 1).is_err());
    }

    #[test]
    fn test_validate_template() {
        assert!(validate_template("alpine").is_ok());
        assert!(validate_template("ubuntu-20-04").is_ok());

        assert!(validate_template("").is_err());
        assert!(validate_template("Ubuntu").is_err());
        assert!(validate_template(&"a".repeat(33)).is_err());
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, Connection, ConnectionGroup, DatabaseType};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_yaml_config_serialization() {
        let mut config = Config::default();

        let mut connection = Connection::new(
            "test_id".to_string(),
            "Test Connection".to_string(),
            DatabaseType::PostgreSQL,
            "localhost".to_string(),
            5432,
            "test_user".to_string(),
            Some("test_db".to_string()),
        );

        connection.set_password("secret_password").unwrap();
        connection.tags = vec!["test".to_string(), "local".to_string()];

        config.add_connection(connection);

        let group = ConnectionGroup {
            name: "Test Group".to_string(),
            connections: vec!["test_id".to_string()],
            description: Some("Test description".to_string()),
        };
        config.add_connection_group(group);

        config.default_connection_group = Some("Test Group".to_string());

        let yaml_content = serde_yaml::to_string(&config).unwrap();
        println!("YAML output:\n{}", yaml_content);

        let deserialized: Config = serde_yaml::from_str(&yaml_content).unwrap();

        assert_eq!(deserialized.connections.len(), 1);
        assert_eq!(deserialized.connections[0].id, "test_id");
        assert_eq!(deserialized.connections[0].name, "Test Connection");
        assert_eq!(deserialized.connections[0].tags.len(), 2);
        assert!(deserialized.connections[0].secure_password.is_some());

        assert_eq!(deserialized.connection_groups.len(), 1);
        assert_eq!(deserialized.connection_groups[0].name, "Test Group");
        assert_eq!(
            deserialized.default_connection_group,
            Some("Test Group".to_string())
        );
    }

    #[test]
    fn test_password_security() {
        let mut connection = Connection::new(
            "test_id".to_string(),
            "Test Connection".to_string(),
            DatabaseType::PostgreSQL,
            "localhost".to_string(),
            5432,
            "test_user".to_string(),
            None,
        );

        let password = "my_secret_password_123";
        connection.set_password(password).unwrap();

        assert!(connection.verify_password(password).unwrap());
        assert!(!connection.verify_password("wrong_password").unwrap());

        let yaml_content = serde_yaml::to_string(&connection).unwrap();
        assert!(!yaml_content.contains(password));
        println!("Secure connection YAML:\n{}", yaml_content);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        let invalid_connection = Connection {
            id: "".to_string(),
            name: "Valid Name".to_string(),
            database_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            username: "user".to_string(),
            secure_password: None,
            database_name: None,
            tags: Vec::new(),
        };
        config.add_connection(invalid_connection);

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_connection_group_validation() {
        let mut config = Config::default();

        let connection = Connection::new(
            "valid_id".to_string(),
            "Valid Connection".to_string(),
            DatabaseType::PostgreSQL,
            "localhost".to_string(),
            5432,
            "user".to_string(),
            None,
        );
        config.add_connection(connection);

        let invalid_group = ConnectionGroup {
            name: "Test Group".to_string(),
            connections: vec!["nonexistent_id".to_string()],
            description: None,
        };
        config.add_connection_group(invalid_group);

        assert!(config.validate().is_err());

        config.connection_groups.clear();
        let valid_group = ConnectionGroup {
            name: "Test Group".to_string(),
            connections: vec!["valid_id".to_string()],
            description: None,
        };
        config.add_connection_group(valid_group);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_yaml_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        let mut config = Config::default();
        let mut connection = Connection::new(
            "file_test".to_string(),
            "File Test Connection".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "test_user".to_string(),
            Some("test_db".to_string()),
        );
        connection.set_password("file_test_password").unwrap();
        config.add_connection(connection);

        let yaml_content = serde_yaml::to_string(&config).unwrap();
        fs::write(&config_path, yaml_content).unwrap();

        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = serde_yaml::from_str(&loaded_content).unwrap();

        assert_eq!(loaded_config.connections.len(), 1);
        assert_eq!(loaded_config.connections[0].id, "file_test");
        assert!(
            loaded_config.connections[0]
                .verify_password("file_test_password")
                .unwrap()
        );
    }
}

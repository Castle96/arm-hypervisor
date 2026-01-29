#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePool;
    use crate::container_store::ContainerStore;
    use models::{ContainerConfig, ContainerStatus};
    use crate::pool::{create_pool, PoolConfig};

    async fn setup_test_db() -> SqlitePool {
        let config = PoolConfig {
            database_url: "sqlite://:memory:".to_string(),
            max_connections: 5,
        };
        create_pool(config).await.expect("Failed to create pool")
    }

    #[tokio::test]
    async fn test_container_store_get_or_create() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(2),
            memory_limit: Some(512 * 1024 * 1024),
            disk_limit: Some(5 * 1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let container = store
            .get_or_create("test-container", "alpine", config)
            .await
            .expect("Failed to create container");

        assert_eq!(container.name, "test-container");
        assert_eq!(container.template, "alpine");
        assert_eq!(container.status, ContainerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_container_store_get_by_name() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let created = store
            .get_or_create("lookup-test", "alpine", config.clone())
            .await
            .expect("Failed to create");

        let retrieved = store
            .get_by_name("lookup-test")
            .await
            .expect("Failed to retrieve");

        assert_eq!(created.id, retrieved.id);
        assert_eq!(created.name, retrieved.name);
    }

    #[tokio::test]
    async fn test_container_store_list() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        // Create multiple containers
        for i in 0..3 {
            let _ = store
                .get_or_create(&format!("test-{}", i), "alpine", config.clone())
                .await;
        }

        let containers = store.list().await.expect("Failed to list");
        assert_eq!(containers.len(), 3);
    }

    #[tokio::test]
    async fn test_container_store_update_status() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let _ = store
            .get_or_create("status-test", "alpine", config)
            .await
            .expect("Failed to create");

        store
            .update_status("status-test", "running")
            .await
            .expect("Failed to update");

        let container = store
            .get_by_name("status-test")
            .await
            .expect("Failed to get");

        assert_eq!(container.status, ContainerStatus::Running);
    }

    #[tokio::test]
    async fn test_container_store_delete() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let _ = store
            .get_or_create("delete-test", "alpine", config)
            .await
            .expect("Failed to create");

        store
            .delete("delete-test")
            .await
            .expect("Failed to delete");

        let result = store.get_by_name("delete-test").await;
        assert!(result.is_err(), "Container should not exist after deletion");
    }

    #[tokio::test]
    async fn test_container_store_exists() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let _ = store
            .get_or_create("exists-test", "alpine", config)
            .await;

        assert!(
            store
                .exists("exists-test")
                .await
                .expect("Failed to check"),
            "Container should exist"
        );

        assert!(
            !store
                .exists("nonexistent")
                .await
                .expect("Failed to check"),
            "Nonexistent container should not exist"
        );
    }

    #[tokio::test]
    async fn test_container_store_get_or_create_idempotent() {
        let pool = setup_test_db().await;
        let store = ContainerStore::new(pool);

        let config = ContainerConfig {
            cpu_limit: Some(1),
            memory_limit: Some(256 * 1024 * 1024),
            disk_limit: Some(1024 * 1024 * 1024),
            network_interfaces: vec![],
            rootfs_path: "/var/lib/lxc/test/rootfs".to_string(),
            environment: vec![],
        };

        let container1 = store
            .get_or_create("idempotent-test", "alpine", config.clone())
            .await
            .expect("Failed to create");

        // Call again with same parameters
        let container2 = store
            .get_or_create("idempotent-test", "alpine", config)
            .await
            .expect("Failed to get");

        // IDs should be identical
        assert_eq!(
            container1.id, container2.id,
            "get_or_create should return same container"
        );
    }
}

mod helpers;

#[cfg(test)]
mod caches {
    use crate::helpers::*;
    use http::StatusCode;
    use infinispan_fork::request::caches::modes::*;
    use infinispan_fork::request::caches::Cache;
    use infinispan_fork::request::{caches, entries};
    use reqwest::Response;
    use serde_json::Value;
    use serial_test::serial;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[tokio::test]
    #[serial]
    async fn create_local() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_local(cache_name)).await;

        assert_eq!(
            get_cache_config(cache_name).await,
            Cache::Local(Local::default())
        );
    }

    #[tokio::test]
    #[serial]
    async fn create_replicated_async() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_replicated_async(cache_name)).await;

        assert_eq!(
            get_cache_config(cache_name).await,
            Cache::Replicated(Replicated::create_async())
        );
    }

    #[tokio::test]
    #[serial]
    async fn create_replicated_sync() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_replicated_sync(cache_name)).await;

        assert_eq!(
            get_cache_config(cache_name).await,
            Cache::Replicated(Replicated::create_sync())
        );
    }

    #[tokio::test]
    #[serial]
    async fn create_distributed_async() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_distributed_async(cache_name)).await;

        assert_eq!(
            get_cache_config(cache_name).await,
            Cache::Distributed(Distributed::create_async())
        );
    }

    #[tokio::test]
    #[serial]
    async fn create_distributed_sync() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_distributed_sync(cache_name)).await;

        assert_eq!(
            get_cache_config(cache_name).await,
            Cache::Distributed(Distributed::create_sync())
        );
    }

    #[tokio::test]
    #[serial]
    async fn create_invalidation_async() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_invalidation_async(cache_name)).await;

        assert_eq!(
            get_cache_config(cache_name).await,
            Cache::Invalidation(Invalidation::create_async())
        );
    }

    #[tokio::test]
    #[serial]
    async fn create_invalidation_sync() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_invalidation_sync(cache_name)).await;

        assert_eq!(
            get_cache_config(cache_name).await,
            Cache::Invalidation(Invalidation::create_sync())
        );
    }

    #[tokio::test]
    #[serial]
    async fn get() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_local(cache_name)).await;

        let resp = run(&caches::get(cache_name)).await;
        let info: Value = serde_json::from_str(&read_body(resp).await).unwrap();

        // Basic checks
        assert!(!info["stats"].is_null());
        assert!(!info["configuration"].is_null());
    }

    #[tokio::test]
    #[serial]
    async fn get_config() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_local(cache_name)).await;

        let resp = run(&caches::get_config(cache_name)).await;
        let config: Value = serde_json::from_str(&read_body(resp).await).unwrap();

        // Basic check
        assert!(!config["local-cache"].is_null());
    }

    #[tokio::test]
    #[serial]
    async fn delete() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_local(cache_name)).await;

        let _ = run(&caches::delete(cache_name)).await;
        let resp = run(&caches::exists(cache_name)).await;

        assert_eq!(StatusCode::NOT_FOUND, resp.status());
    }

    #[tokio::test]
    #[serial]
    async fn get_keys() {
        cleanup().await;

        let cache_name = "test_cache";
        let keys: HashSet<String> = HashSet::from_iter(vec!["k1".into(), "k2".into()]);

        let _ = run(&caches::create_local(cache_name)).await;

        for key in &keys {
            let _ = run(&entries::create(cache_name, key)).await;
        }

        let resp = run(&caches::keys(cache_name)).await;

        assert_eq!(keys, serde_json::from_str(&read_body(resp).await).unwrap())
    }

    #[tokio::test]
    #[serial]
    async fn clear() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_local(cache_name)).await;
        let _ = run(&entries::create(cache_name, "some_entry")).await;

        let _ = run(&caches::clear(cache_name)).await;
        let resp = run(&caches::size(cache_name)).await;

        assert_eq!("0", read_body(resp).await);
    }

    #[tokio::test]
    #[serial]
    async fn size() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_local(cache_name)).await;

        let resp = run(&caches::size(cache_name)).await;
        assert_eq!("0", read_body(resp).await);

        let _ = run(&entries::create(cache_name, "some_entry")).await;
        let resp = run(&caches::size(cache_name)).await;
        assert_eq!("1", read_body(resp).await);
    }

    #[tokio::test]
    #[serial]
    async fn stats() {
        cleanup().await;

        let cache_name = "test_cache";

        let _ = run(&caches::create_local(cache_name)).await;

        let resp = run(&caches::stats(cache_name)).await;
        let config: Value = serde_json::from_str(&read_body(resp).await).unwrap();

        // Basic check
        assert!(!config["time_since_start"].is_null());
        assert!(!config["time_since_reset"].is_null());
    }

    #[tokio::test]
    #[serial]
    async fn list() {
        cleanup().await;

        let cache_names: HashSet<String> =
            HashSet::from_iter(vec!["cache_1".into(), "cache_2".into()]);

        for cache_name in &cache_names {
            let _ = run(&caches::create_local(cache_name)).await;
        }

        let resp = run(&caches::list()).await;

        assert_eq!(cache_names, cache_names_from_list_resp(resp).await);
    }

    async fn cleanup() {
        let resp = run(&caches::list()).await;

        for counter_name in cache_names_from_list_resp(resp).await {
            let _ = run(&caches::delete(counter_name)).await;
        }
    }

    async fn cache_names_from_list_resp(response: Response) -> HashSet<String> {
        serde_json::from_str(&read_body(response).await).unwrap()
    }

    async fn get_cache_config(name: impl AsRef<str>) -> Cache {
        let resp = run(&caches::get_config(name)).await;
        serde_json::from_str::<Cache>(&read_body(resp).await).unwrap()
    }
}

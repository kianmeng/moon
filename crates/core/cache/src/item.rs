#[macro_export]
macro_rules! cache_item {
    ($struct:ident) => {
        impl $struct {
            pub async fn load(path: PathBuf, stale_ms: u128) -> Result<Self, MoonError> {
                let mut item = Self::default();
                let log_target = "moon:cache:item";

                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                if is_readable() {
                    if path.exists() {
                        // If stale, treat as a cache miss
                        if stale_ms > 0
                            && time::now_millis()
                                - time::to_millis(fs::metadata(&path).await?.modified().unwrap())
                                > stale_ms
                        {
                            trace!(
                                target: log_target,
                                "Cache skip for {}, marked as stale",
                                color::path(&path)
                            );
                        } else {
                            trace!(
                                target: log_target,
                                "Cache hit for {}, reading",
                                color::path(&path)
                            );

                            item = fs::read_json(&path).await?;
                        }
                    } else {
                        trace!(
                            target: log_target,
                            "Cache miss for {}, does not exist",
                            color::path(&path)
                        );
                    }
                }

                item.path = path;

                Ok(item)
            }

            pub async fn save(&self) -> Result<(), MoonError> {
                let log_target = "moon:cache:item";

                if is_writable() {
                    trace!(
                        target: log_target,
                        "Writing cache {}",
                        color::path(&self.path)
                    );

                    fs::write_json(&self.path, &self, false).await?;
                }

                Ok(())
            }

            pub fn get_dir(&self) -> &Path {
                self.path.parent().unwrap()
            }
        }
    };
}

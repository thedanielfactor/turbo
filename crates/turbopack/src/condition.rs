use async_recursion::async_recursion;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use turbo_tasks::trace::TraceRawVcs;
use turbo_tasks_fs::{FileSystem, FileSystemPath, FileSystemPathVc};

#[derive(Debug, Clone, Serialize, Deserialize, TraceRawVcs, PartialEq, Eq)]
pub enum ContextCondition {
    All(Vec<ContextCondition>),
    Any(Vec<ContextCondition>),
    Not(Box<ContextCondition>),
    InDirectory(String),
    InPath(FileSystemPathVc),
}

impl ContextCondition {
    /// Creates a condition that matches if all of the given conditions match.
    pub fn all(conditions: Vec<ContextCondition>) -> ContextCondition {
        ContextCondition::All(conditions)
    }

    /// Creates a condition that matches if any of the given conditions match.
    pub fn any(conditions: Vec<ContextCondition>) -> ContextCondition {
        ContextCondition::Any(conditions)
    }

    /// Creates a condition that matches if the given condition does not match.
    #[allow(clippy::should_implement_trait)]
    pub fn not(condition: ContextCondition) -> ContextCondition {
        ContextCondition::Not(Box::new(condition))
    }

    #[async_recursion]
    /// Returns true if the condition matches the context.
    pub async fn matches(&self, context: &FileSystemPath) -> bool {
        match self {
            ContextCondition::All(conditions) => {
                stream::iter(conditions)
                    .all(|c| async move { c.matches(context).await })
                    .await
            }
            ContextCondition::Any(conditions) => {
                stream::iter(conditions)
                    .any(|c| async move { c.matches(context).await })
                    .await
            }
            ContextCondition::Not(condition) => !condition.matches(context).await,
            ContextCondition::InPath(path) => {
                if let Ok(path) = path.await {
                    path.fs
                        .root()
                        .await
                        .map_or(false, |root| context.is_inside(&root))
                } else {
                    false
                }
            }
            ContextCondition::InDirectory(dir) => {
                context.path.starts_with(&format!("{dir}/"))
                    || context.path.contains(&format!("/{dir}/"))
                    || context.path.ends_with(&format!("/{dir}"))
                    || context.path == *dir
            }
        }
    }
}

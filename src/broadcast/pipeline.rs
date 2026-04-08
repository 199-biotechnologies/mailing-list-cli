//! Broadcast send pipeline. Full impl in Task 6.

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum PipelineError {
    #[error("pipeline error: {0}")]
    Generic(String),
}

#[allow(dead_code)]
pub struct PipelineResult {
    pub sent_count: usize,
    pub suppressed_count: usize,
    pub failed_count: usize,
}

#[allow(dead_code)]
pub fn send_broadcast(_id: i64) -> Result<PipelineResult, crate::error::AppError> {
    Err(crate::error::AppError::Transient {
        code: "pipeline_not_implemented".into(),
        message: "send_broadcast not yet implemented".into(),
        suggestion: "Task 6 implements this".into(),
    })
}

#[allow(dead_code)]
pub fn preview_broadcast(
    _id: i64,
    _to: &str,
) -> Result<PipelineResult, crate::error::AppError> {
    Err(crate::error::AppError::Transient {
        code: "preview_not_implemented".into(),
        message: "preview_broadcast not yet implemented".into(),
        suggestion: "Task 6 implements this".into(),
    })
}

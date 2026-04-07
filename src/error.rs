/// Semantic exit codes per the agent-cli-framework contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    Transient = 1,
    Config = 2,
    BadInput = 3,
    #[allow(dead_code)]
    RateLimited = 4,
}

impl ExitCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("transient error: {message}")]
    Transient {
        code: String,
        message: String,
        suggestion: String,
    },
    #[error("config error: {message}")]
    Config {
        code: String,
        message: String,
        suggestion: String,
    },
    #[error("bad input: {message}")]
    #[allow(dead_code)]
    BadInput {
        code: String,
        message: String,
        suggestion: String,
    },
    #[error("rate limited: {message}")]
    #[allow(dead_code)]
    RateLimited {
        code: String,
        message: String,
        suggestion: String,
    },
}

impl AppError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            AppError::Transient { .. } => ExitCode::Transient,
            AppError::Config { .. } => ExitCode::Config,
            AppError::BadInput { .. } => ExitCode::BadInput,
            AppError::RateLimited { .. } => ExitCode::RateLimited,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AppError::Transient { code, .. }
            | AppError::Config { code, .. }
            | AppError::BadInput { code, .. }
            | AppError::RateLimited { code, .. } => code,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            AppError::Transient { message, .. }
            | AppError::Config { message, .. }
            | AppError::BadInput { message, .. }
            | AppError::RateLimited { message, .. } => message,
        }
    }

    pub fn suggestion(&self) -> &str {
        match self {
            AppError::Transient { suggestion, .. }
            | AppError::Config { suggestion, .. }
            | AppError::BadInput { suggestion, .. }
            | AppError::RateLimited { suggestion, .. } => suggestion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::Transient.as_i32(), 1);
        assert_eq!(ExitCode::Config.as_i32(), 2);
        assert_eq!(ExitCode::BadInput.as_i32(), 3);
        assert_eq!(ExitCode::RateLimited.as_i32(), 4);
    }

    #[test]
    fn config_error_has_exit_code_2() {
        let err = AppError::Config {
            code: "missing_email_cli".into(),
            message: "email-cli not on PATH".into(),
            suggestion: "Install email-cli with `brew install email-cli`".into(),
        };
        assert_eq!(err.exit_code(), ExitCode::Config);
        assert_eq!(err.code(), "missing_email_cli");
    }
}

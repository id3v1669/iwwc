use crate::data::nf_struct::NotificationAction;

pub enum ErrorHandler {
    ZbusFdo(zbus::fdo::Error),
    NotificationSend(tokio::sync::mpsc::error::SendError<NotificationAction>),
    CStr(std::ffi::NulError),
    Other { message: String },
}

impl From<zbus::Error> for ErrorHandler {
    fn from(err: zbus::Error) -> Self {
        match err {
            zbus::Error::FDO(e) => Self::ZbusFdo(*e),
            _ => Self::Other {
                message: err.to_string(),
            },
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<NotificationAction>> for ErrorHandler {
    fn from(err: tokio::sync::mpsc::error::SendError<NotificationAction>) -> Self {
        ErrorHandler::NotificationSend(err)
    }
}

impl From<std::ffi::NulError> for ErrorHandler {
    fn from(err: std::ffi::NulError) -> Self {
        ErrorHandler::CStr(err)
    }
}

impl From<ErrorHandler> for zbus::fdo::Error {
    fn from(err: ErrorHandler) -> Self {
        match err {
            ErrorHandler::ZbusFdo(e) => e,
            ErrorHandler::Other { message } => zbus::fdo::Error::Failed(message),
            ErrorHandler::NotificationSend(e) => zbus::fdo::Error::Failed(e.to_string()),
            ErrorHandler::CStr(e) => zbus::fdo::Error::Failed(e.to_string()),
        }
    }
}

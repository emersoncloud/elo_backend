use rocket::http::Status;
use rocket::response::Responder;
use rocket::Request;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("sqlx error")]
    MyError(#[from] sqlx::Error),
}

impl<'r> Responder<'r, 'static> for ServerError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::MyError(_) => Err(Status::NotFound),
        }
    }
}

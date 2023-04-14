use std::env::VarError;
use salvo::{async_trait, Depot, Request, Response, Writer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing or invalid query parameter: {0}")]
    MissingParameter(&'static str),

    #[error("this coordinates are probably not on earth: lat {0}, lng {1}")]
    NotOnEarth(f32, f32),

    #[error("the city with index {0} was not found")]
    CityNotFound(usize),

    #[error("invalid configuration")]
    ConfigurationError(#[from] VarError)
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.render(self.to_string());
    }
}